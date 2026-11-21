from contextlib import asynccontextmanager
from pathlib import Path

from fastapi import FastAPI
from fastapi.staticfiles import StaticFiles

from app import db
from app.api.routes import liabilities, scenarios
from app.config import settings


@asynccontextmanager
async def lifespan(app: FastAPI):
    await db.init_db(settings.mongodb_uri, settings.mongodb_db)
    yield
    await db.close_db()


app = FastAPI(title="Client Liabilities API", version="0.1.0", lifespan=lifespan)
app.include_router(liabilities.router, prefix="/api/v1")
app.include_router(scenarios.router, prefix="/api/v1")


@app.get("/health", tags=["health"])
async def health():
    await db.ping()
    return {"status": "ok"}


class NoCacheStaticFiles(StaticFiles):
    """Always revalidate, so harness edits show up without a hard refresh."""

    async def get_response(self, path, scope):
        response = await super().get_response(path, scope)
        response.headers["Cache-Control"] = "no-cache"
        return response


# Browser test harness (same origin as the API, so no CORS needed).
app.mount(
    "/harness",
    NoCacheStaticFiles(directory=Path(__file__).resolve().parent.parent / "harness", html=True),
    name="harness",
)
