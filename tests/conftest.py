import pytest_asyncio
from httpx import ASGITransport, AsyncClient

from app import db
from app.config import settings
from app.main import app

TEST_DB = "liabilities_test"
ADVISOR = {"X-Caller-Type": "advisor", "X-User-Id": "adv-1"}
CLIENT = {"X-Caller-Type": "client", "X-User-Id": "cli-1"}


@pytest_asyncio.fixture
async def http():
    client = await db.init_db(settings.mongodb_uri, TEST_DB)
    await client.drop_database(TEST_DB)
    await db.init_db(settings.mongodb_uri, TEST_DB)  # recreate indexes
    async with AsyncClient(transport=ASGITransport(app=app), base_url="http://test") as c:
        yield c
    await client.drop_database(TEST_DB)
    await db.close_db()
