"""Static metadata about the seeded sample clients, used by the browser test harness."""

from fastapi import APIRouter
from pydantic import BaseModel

from app.seed import SCENARIOS, scenario_counts

router = APIRouter(prefix="/seed-scenarios", tags=["seed"])


class ScenarioCounts(BaseModel):
    liabilities: int
    unchanged: int
    edits: int
    adds: int
    deletes: int


class Scenario(BaseModel):
    client_id: str
    title: str
    description: str
    counts: ScenarioCounts


@router.get("", response_model=list[Scenario])
async def list_scenarios() -> list[Scenario]:
    return [
        Scenario(
            client_id=client_id,
            title=title,
            description=description,
            counts=ScenarioCounts(**scenario_counts(items)),
        )
        for client_id, (title, description, items) in SCENARIOS.items()
    ]
