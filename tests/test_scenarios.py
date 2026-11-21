from app.seed import SCENARIOS


async def test_scenarios_listed_without_headers(http):
    r = await http.get("/api/v1/seed-scenarios")
    assert r.status_code == 200
    body = r.json()
    assert [s["client_id"] for s in body] == list(SCENARIOS)
    assert all(s["title"] and s["description"] for s in body)


async def test_scenario_counts(http):
    body = {s["client_id"]: s["counts"] for s in (await http.get("/api/v1/seed-scenarios")).json()}
    assert body["client-family-homeowners"] == {
        "liabilities": 5,
        "unchanged": 1,
        "edits": 2,
        "adds": 1,
        "deletes": 1,
    }
    assert body["client-single-add"] == {
        "liabilities": 1,
        "unchanged": 0,
        "edits": 0,
        "adds": 1,
        "deletes": 0,
    }
    assert body["client-no-pending"]["edits"] == 0
    assert body["client-empty"]["liabilities"] == 0
