from datetime import UTC, date, datetime
from decimal import Decimal
from enum import StrEnum
from typing import Annotated, Any

from beanie import Document
from bson.decimal128 import Decimal128
from pydantic import BaseModel, Field, field_validator, model_validator
from pymongo import ASCENDING, IndexModel


class CallerType(StrEnum):
    advisor = "advisor"
    client = "client"


class LiabilityType(StrEnum):
    mortgage = "mortgage"
    home_equity_loan = "home_equity_loan"
    heloc = "heloc"
    auto_loan = "auto_loan"
    student_loan = "student_loan"
    credit_card = "credit_card"
    personal_loan = "personal_loan"
    medical_debt = "medical_debt"
    business_loan = "business_loan"
    tax_liability = "tax_liability"
    other = "other"


class Ownership(StrEnum):
    client = "client"
    spouse = "spouse"
    joint = "joint"


class RateType(StrEnum):
    fixed = "fixed"
    variable = "variable"


class PaymentFrequency(StrEnum):
    weekly = "weekly"
    biweekly = "biweekly"
    monthly = "monthly"
    quarterly = "quarterly"
    annually = "annually"


# Payments per year, used for normalizing payments in the summary.
PAYMENTS_PER_YEAR: dict[PaymentFrequency, int] = {
    PaymentFrequency.weekly: 52,
    PaymentFrequency.biweekly: 26,
    PaymentFrequency.monthly: 12,
    PaymentFrequency.quarterly: 4,
    PaymentFrequency.annually: 1,
}

Money = Annotated[Decimal, Field(ge=0, max_digits=14, decimal_places=2)]
Last4 = Annotated[str, Field(pattern=r"^\d{4}$")]


def utcnow() -> datetime:
    return datetime.now(UTC)


class AuditStamp(BaseModel):
    user_id: str
    caller_type: CallerType
    at: datetime = Field(default_factory=utcnow)


class LiabilityFields(BaseModel):
    """Fields shared by the document and the create schema."""

    liability_type: LiabilityType
    name: str = Field(min_length=1, max_length=200)
    lender: str | None = Field(default=None, max_length=200)
    account_last4: Last4 | None = None
    owner: Ownership = Ownership.client
    original_balance: Money | None = None
    current_balance: Money
    interest_rate: Annotated[Decimal, Field(ge=0, le=100, decimal_places=4)] | None = None
    rate_type: RateType | None = None
    payment_amount: Money | None = None
    payment_frequency: PaymentFrequency | None = None
    origination_date: date | None = None
    maturity_date: date | None = None
    is_tax_deductible: bool = False
    notes: str | None = Field(default=None, max_length=2000)

    @field_validator(
        "original_balance", "current_balance", "interest_rate", "payment_amount", mode="before"
    )
    @classmethod
    def _from_decimal128(cls, v):
        # MongoDB returns Decimal128; convert back to Decimal on read.
        return v.to_decimal() if isinstance(v, Decimal128) else v

    @model_validator(mode="after")
    def _check_dates(self):
        if (
            self.origination_date
            and self.maturity_date
            and self.maturity_date < self.origination_date
        ):
            raise ValueError("maturity_date must be on or after origination_date")
        return self


class StagedAction(StrEnum):
    create = "create"
    update = "update"
    delete = "delete"


class StagedChange(BaseModel):
    """A client's proposed add, edit or delete, held until an advisor reviews it."""

    action: StagedAction = StagedAction.update
    # For updates: sparse field -> proposed value, stored JSON-mode (Decimal as str, date as
    # ISO str). Empty for create and delete.
    changes: dict[str, Any] = Field(default_factory=dict)
    created_by: AuditStamp  # first client edit
    updated_by: AuditStamp  # latest client edit


class Liability(Document, LiabilityFields):
    client_id: str
    created_at: datetime = Field(default_factory=utcnow)
    updated_at: datetime = Field(default_factory=utcnow)
    created_by: AuditStamp
    updated_by: AuditStamp
    staged: StagedChange | None = None

    class Settings:
        name = "liabilities"
        indexes = [
            IndexModel([("client_id", ASCENDING)]),
            IndexModel([("client_id", ASCENDING), ("liability_type", ASCENDING)]),
        ]
