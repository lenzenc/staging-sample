from beanie import init_beanie
from pymongo import AsyncMongoClient

from app.models.liability import Liability

_client: AsyncMongoClient | None = None


async def init_db(uri: str, db_name: str) -> AsyncMongoClient:
    global _client
    _client = AsyncMongoClient(uri, tz_aware=True)
    await init_beanie(database=_client[db_name], document_models=[Liability])
    return _client


async def close_db() -> None:
    global _client
    if _client is not None:
        await _client.close()
        _client = None


async def ping() -> None:
    assert _client is not None
    await _client.admin.command("ping")
