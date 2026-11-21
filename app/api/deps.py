from dataclasses import dataclass
from typing import Annotated

from fastapi import Depends, Header

from app.models.liability import AuditStamp, CallerType


@dataclass(frozen=True)
class CallerContext:
    caller_type: CallerType
    user_id: str

    def stamp(self) -> AuditStamp:
        return AuditStamp(user_id=self.user_id, caller_type=self.caller_type)


def get_caller(
    x_caller_type: Annotated[CallerType, Header(description="advisor or client app")],
    x_user_id: Annotated[str, Header(min_length=1, description="Calling user's ID")],
) -> CallerContext:
    return CallerContext(caller_type=x_caller_type, user_id=x_user_id)


Caller = Annotated[CallerContext, Depends(get_caller)]
