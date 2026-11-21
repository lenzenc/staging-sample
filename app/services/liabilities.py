from collections import defaultdict
from decimal import Decimal
from typing import Any

from beanie import PydanticObjectId
from bson.errors import InvalidId
from pydantic import ValidationError
from pydantic_core import to_jsonable_python

from app.api.deps import CallerContext
from app.models.liability import (
    PAYMENTS_PER_YEAR,
    CallerType,
    Liability,
    LiabilityType,
    StagedAction,
    StagedChange,
    utcnow,
)
from app.schemas.liability import (
    ClientLiabilityRead,
    FieldChange,
    LiabilityCreate,
    LiabilityList,
    LiabilityRead,
    LiabilitySummary,
    LiabilityUpdate,
    PendingChange,
)


def _staged_values(doc: Liability) -> dict[str, Any]:
    """The staged edits as typed values (Decimal, date, enum), keeping explicit nulls."""
    if doc.staged is None or doc.staged.action != StagedAction.update:
        return {}
    return LiabilityUpdate.model_validate(doc.staged.changes).model_dump(exclude_unset=True)


def _effective(doc: Liability) -> Liability:
    """The liability as a client sees it: committed values overlaid with staged edits."""
    staged = _staged_values(doc)
    if not staged:
        return doc
    return Liability.model_validate({**doc.model_dump(), **staged})


def _pending(doc: Liability) -> PendingChange | None:
    if doc.staged is None:
        return None
    staged = _staged_values(doc)
    if doc.staged.action == StagedAction.update and not staged:
        return None
    return PendingChange(
        action=doc.staged.action,
        changes={
            field: FieldChange(
                current=to_jsonable_python(getattr(doc, field)),
                proposed=to_jsonable_python(value),
            )
            for field, value in staged.items()
        },
        created_by=doc.staged.created_by,
        updated_by=doc.staged.updated_by,
    )


def to_read(doc: Liability, caller: CallerContext) -> ClientLiabilityRead | LiabilityRead:
    """Clients get their updated view (committed + staged values) with no change info.

    Advisors get the committed values plus `pending_change` describing what is staged.
    """
    exclude = {"id", "revision_id", "staged"}
    if caller.caller_type == CallerType.client:
        return ClientLiabilityRead(id=str(doc.id), **_effective(doc).model_dump(exclude=exclude))
    return LiabilityRead(
        id=str(doc.id), pending_change=_pending(doc), **doc.model_dump(exclude=exclude)
    )


def _action(doc: Liability) -> StagedAction | None:
    return doc.staged.action if doc.staged else None


def _hidden_from(doc: Liability, caller: CallerContext) -> bool:
    """A client's pending delete hides the record from that client."""
    return caller.caller_type == CallerType.client and _action(doc) == StagedAction.delete


def _parse_id(liability_id: str) -> PydanticObjectId | None:
    try:
        return PydanticObjectId(liability_id)
    except InvalidId, TypeError:
        return None


async def create(client_id: str, body: LiabilityCreate, caller: CallerContext) -> Liability:
    stamp = caller.stamp()
    # A client's new liability is a proposal until an advisor reviews it.
    staged = (
        StagedChange(action=StagedAction.create, created_by=stamp, updated_by=stamp)
        if caller.caller_type == CallerType.client
        else None
    )
    doc = Liability(
        client_id=client_id,
        created_by=stamp,
        updated_by=stamp,
        staged=staged,
        **body.model_dump(),
    )
    await doc.insert()
    return doc


async def list_for_client(
    client_id: str,
    liability_type: LiabilityType | None,
    skip: int,
    limit: int,
    caller: CallerContext,
) -> LiabilityList:
    query = Liability.find(Liability.client_id == client_id)
    if caller.caller_type == CallerType.client:
        query = query.find({"staged.action": {"$ne": StagedAction.delete.value}})
    if liability_type:
        if caller.caller_type == CallerType.client:
            # Filter on the type the client sees (staged value wins over committed).
            staged_type = "staged.changes.liability_type"
            query = query.find(
                {
                    "$or": [
                        {"liability_type": liability_type.value, staged_type: {"$exists": False}},
                        {staged_type: liability_type.value},
                    ]
                }
            )
        else:
            query = query.find(Liability.liability_type == liability_type)
    total = await query.count()
    docs = await query.sort("-created_at").skip(skip).limit(limit).to_list()
    return LiabilityList(items=[to_read(d, caller) for d in docs], total=total)


async def get(client_id: str, liability_id: str, caller: CallerContext) -> Liability | None:
    oid = _parse_id(liability_id)
    if oid is None:
        return None
    doc = await Liability.find_one(Liability.id == oid, Liability.client_id == client_id)
    return None if doc is None or _hidden_from(doc, caller) else doc


async def update(
    client_id: str, liability_id: str, body: LiabilityUpdate, caller: CallerContext
) -> Liability | None:
    """Advisor edits apply immediately; client edits are staged for advisor review."""
    doc = await get(client_id, liability_id, caller)
    if doc is None:
        return None
    changes = body.model_dump(exclude_unset=True)
    if caller.caller_type == CallerType.client and _action(doc) != StagedAction.create:
        await _stage(doc, changes, caller)
    else:
        await _apply(doc, changes, caller)
    return doc


def _store_staged(doc: Liability, values: dict[str, Any], caller: CallerContext) -> None:
    """Set doc.staged from typed values; clear it when nothing is left."""
    if not values:
        doc.staged = None
        return
    stamp = caller.stamp()
    doc.staged = StagedChange(
        action=StagedAction.update,
        changes=to_jsonable_python(values),
        created_by=doc.staged.created_by if doc.staged else stamp,
        updated_by=stamp,
    )


async def _stage(doc: Liability, changes: dict[str, Any], caller: CallerContext) -> None:
    combined = {**_staged_values(doc), **changes}
    # Setting a field back to its committed value un-stages it.
    combined = {k: v for k, v in combined.items() if getattr(doc, k) != v}
    # Validate the would-be record (e.g. date ordering); raises ValidationError -> 422.
    Liability.model_validate({**doc.model_dump(), **combined})
    _store_staged(doc, combined, caller)
    await doc.save()


async def _apply(doc: Liability, changes: dict[str, Any], caller: CallerContext) -> None:
    merged = {**doc.model_dump(), **changes}
    # Re-validate the combined document (e.g. date ordering).
    validated = Liability.model_validate(merged)
    for field in changes:
        setattr(doc, field, getattr(validated, field))
    doc.updated_at = utcnow()
    doc.updated_by = caller.stamp()
    # The advisor's edit wins: drop staged fields it overlaps or that now match the record.
    remaining = {
        k: v for k, v in _staged_values(doc).items() if k not in changes and getattr(doc, k) != v
    }
    if doc.staged is not None and doc.staged.action != StagedAction.update:
        # Pending add/delete: the record changes, the flag stays (client edits to a pending
        # add land here too and just move the latest-edit stamp).
        if caller.caller_type == CallerType.client:
            doc.staged = doc.staged.model_copy(update={"updated_by": caller.stamp()})
        await doc.save()
        return
    if remaining:
        try:
            Liability.model_validate({**doc.model_dump(), **remaining})
        except ValidationError:
            remaining = {}  # leftovers no longer make a valid record
    if doc.staged is not None:
        # Prune only: keep the client's original stamps.
        doc.staged = (
            doc.staged.model_copy(update={"changes": to_jsonable_python(remaining)})
            if remaining
            else None
        )
    await doc.save()


async def delete(client_id: str, liability_id: str, caller: CallerContext) -> bool:
    """Advisors hard-delete. A client's delete is staged, unless the record is their own
    still-pending add (nothing committed to review, so it just goes away)."""
    doc = await get(client_id, liability_id, caller)
    if doc is None:
        return False
    if caller.caller_type == CallerType.client and _action(doc) != StagedAction.create:
        stamp = caller.stamp()
        doc.staged = StagedChange(action=StagedAction.delete, created_by=stamp, updated_by=stamp)
        await doc.save()
        return True
    await doc.delete()
    return True


async def summary(client_id: str, caller: CallerContext) -> LiabilitySummary:
    """Clients get totals over their staged view; advisors over committed data."""
    raw = await Liability.find(Liability.client_id == client_id).to_list()
    advisor = caller.caller_type == CallerType.advisor  # clients get no change info
    pending_by_action: dict[str, int] = defaultdict(int)
    for d in raw:
        if d.staged is not None:
            pending_by_action[d.staged.action.value] += 1
    if caller.caller_type == CallerType.client:  # pending deletes are gone, adds are in
        docs = [_effective(d) for d in raw if _action(d) != StagedAction.delete]
    else:  # pending adds are not committed yet
        docs = [d for d in raw if _action(d) != StagedAction.create]
    count_by_type: dict[str, int] = defaultdict(int)
    balance_by_type: dict[str, Decimal] = defaultdict(Decimal)
    total_balance = Decimal(0)
    annual_payments = Decimal(0)
    for d in docs:
        count_by_type[d.liability_type] += 1
        balance_by_type[d.liability_type] += d.current_balance
        total_balance += d.current_balance
        if d.payment_amount is not None and d.payment_frequency is not None:
            annual_payments += d.payment_amount * PAYMENTS_PER_YEAR[d.payment_frequency]
    return LiabilitySummary(
        client_id=client_id,
        count=len(docs),
        total_current_balance=total_balance,
        count_by_type=dict(count_by_type),
        balance_by_type=dict(balance_by_type),
        total_annual_payments=annual_payments,
        pending_change_count=sum(pending_by_action.values()) if advisor else 0,
        pending_by_action=dict(pending_by_action) if advisor else {},
    )
