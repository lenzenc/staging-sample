from typing import Annotated

from fastapi import APIRouter, HTTPException, Query, Response, status
from pydantic import ValidationError

from app.api.deps import Caller
from app.models.liability import LiabilityType
from app.schemas.liability import (
    ClientLiabilityRead,
    LiabilityCreate,
    LiabilityList,
    LiabilityRead,
    LiabilitySummary,
    LiabilityUpdate,
)
from app.services import liabilities as svc

ReadModel = ClientLiabilityRead | LiabilityRead

router = APIRouter(prefix="/clients/{client_id}/liabilities", tags=["liabilities"])

NOT_FOUND = HTTPException(status.HTTP_404_NOT_FOUND, "Liability not found")


@router.post("", response_model=ReadModel, status_code=status.HTTP_201_CREATED)
async def create_liability(client_id: str, body: LiabilityCreate, caller: Caller):
    return svc.to_read(await svc.create(client_id, body, caller), caller)


@router.get("", response_model=LiabilityList)
async def list_liabilities(
    client_id: str,
    liability_type: LiabilityType | None = None,
    skip: Annotated[int, Query(ge=0)] = 0,
    limit: Annotated[int, Query(ge=1, le=200)] = 50,
    *,
    caller: Caller,
):
    return await svc.list_for_client(client_id, liability_type, skip, limit, caller)


# Declared before "/{liability_id}" so "summary" is not treated as an ID.
@router.get("/summary", response_model=LiabilitySummary)
async def liabilities_summary(client_id: str, caller: Caller):
    return await svc.summary(client_id, caller)


@router.get("/{liability_id}", response_model=ReadModel)
async def get_liability(client_id: str, liability_id: str, caller: Caller):
    doc = await svc.get(client_id, liability_id, caller)
    if doc is None:
        raise NOT_FOUND
    return svc.to_read(doc, caller)


@router.patch("/{liability_id}", response_model=ReadModel)
async def update_liability(
    client_id: str, liability_id: str, body: LiabilityUpdate, caller: Caller
):
    try:
        doc = await svc.update(client_id, liability_id, body, caller)
    except ValidationError as exc:  # merged document failed cross-field validation
        raise HTTPException(
            status.HTTP_422_UNPROCESSABLE_CONTENT,
            exc.errors(include_url=False, include_context=False, include_input=False),
        ) from exc
    if doc is None:
        raise NOT_FOUND
    return svc.to_read(doc, caller)


@router.delete("/{liability_id}", status_code=status.HTTP_204_NO_CONTENT)
async def delete_liability(client_id: str, liability_id: str, caller: Caller):
    if not await svc.delete(client_id, liability_id, caller):
        raise NOT_FOUND
    return Response(status_code=status.HTTP_204_NO_CONTENT)
