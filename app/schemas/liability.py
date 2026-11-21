from datetime import date, datetime
from decimal import Decimal
from typing import Annotated, Any

from pydantic import BaseModel, ConfigDict, Field, model_validator

from app.models.liability import (
    AuditStamp,
    Last4,
    LiabilityFields,
    LiabilityType,
    Money,
    Ownership,
    PaymentFrequency,
    RateType,
    StagedAction,
)


class LiabilityCreate(LiabilityFields):
    pass


class LiabilityUpdate(BaseModel):
    """PATCH body: every field optional; only fields sent are changed."""

    liability_type: LiabilityType | None = None
    name: str | None = Field(default=None, min_length=1, max_length=200)
    lender: str | None = Field(default=None, max_length=200)
    account_last4: Last4 | None = None
    owner: Ownership | None = None
    original_balance: Money | None = None
    current_balance: Money | None = None
    interest_rate: Annotated[Decimal, Field(ge=0, le=100, decimal_places=4)] | None = None
    rate_type: RateType | None = None
    payment_amount: Money | None = None
    payment_frequency: PaymentFrequency | None = None
    origination_date: date | None = None
    maturity_date: date | None = None
    is_tax_deductible: bool | None = None
    notes: str | None = Field(default=None, max_length=2000)

    @model_validator(mode="after")
    def _no_null_required(self):
        for f in ("liability_type", "name", "owner", "current_balance", "is_tax_deductible"):
            if f in self.model_fields_set and getattr(self, f) is None:
                raise ValueError(f"{f} cannot be null")
        return self


class FieldChange(BaseModel):
    current: Any
    proposed: Any


class PendingChange(BaseModel):
    """What a client has staged for a liability, as current -> proposed per field."""

    action: StagedAction
    changes: dict[str, FieldChange] = {}
    created_by: AuditStamp
    updated_by: AuditStamp


class ClientLiabilityRead(LiabilityFields):
    """A liability as a client sees it: committed data merged with their staged changes.

    Carries no change information. `extra="forbid"` makes a payload that has a
    `pending_change` key (the advisor's) fail this model, so unions resolve correctly.
    """

    model_config = ConfigDict(from_attributes=True, extra="forbid")

    id: str
    client_id: str
    created_at: datetime
    updated_at: datetime
    created_by: AuditStamp
    updated_by: AuditStamp


class LiabilityRead(ClientLiabilityRead):
    """The advisor's view: committed data plus what the client has staged."""

    pending_change: PendingChange | None = None


class LiabilityList(BaseModel):
    items: list[ClientLiabilityRead | LiabilityRead]
    total: int


class LiabilitySummary(BaseModel):
    client_id: str
    count: int
    total_current_balance: Decimal
    count_by_type: dict[str, int]
    balance_by_type: dict[str, Decimal]
    total_annual_payments: Decimal
    pending_change_count: int = 0
    pending_by_action: dict[str, int] = {}
