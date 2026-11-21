import pytest

from tests.conftest import ADVISOR

URL = "/api/v1/clients/c1/liabilities"
MORTGAGE = {
    "liability_type": "mortgage",
    "name": "Primary mortgage",
    "current_balance": "250000.00",
    "interest_rate": "6.5",
    "payment_amount": "2000.00",
    "payment_frequency": "monthly",
}
CARD = {"liability_type": "credit_card", "name": "Visa", "current_balance": "1500.50"}


async def _create(http, body=MORTGAGE, headers=ADVISOR, url=URL):
    r = await http.post(url, json=body, headers=headers)
    assert r.status_code == 201, r.text
    return r.json()


async def test_crud_and_audit(http):
    created = await _create(http)
    assert created["created_by"]["caller_type"] == "advisor"
    lid = created["id"]

    other_advisor = {**ADVISOR, "X-User-Id": "adv-2"}
    r = await http.patch(
        f"{URL}/{lid}", json={"current_balance": "240000.00"}, headers=other_advisor
    )
    assert r.status_code == 200
    body = r.json()
    assert body["current_balance"] == "240000.00"
    assert body["name"] == "Primary mortgage"  # untouched
    assert body["created_by"]["user_id"] == "adv-1"
    assert body["updated_by"]["user_id"] == "adv-2"

    assert (await http.get(f"{URL}/{lid}", headers=ADVISOR)).json()["current_balance"] == (
        "240000.00"
    )
    assert (await http.delete(f"{URL}/{lid}", headers=ADVISOR)).status_code == 204
    assert (await http.get(f"{URL}/{lid}", headers=ADVISOR)).status_code == 404


async def test_list_filter_pagination(http):
    await _create(http, MORTGAGE)
    await _create(http, CARD)
    await _create(http, CARD)
    r = (await http.get(URL, headers=ADVISOR)).json()
    assert r["total"] == 3
    r = (await http.get(URL, params={"liability_type": "credit_card"}, headers=ADVISOR)).json()
    assert r["total"] == 2
    r = (await http.get(URL, params={"limit": 1, "skip": 1}, headers=ADVISOR)).json()
    assert r["total"] == 3 and len(r["items"]) == 1


async def test_summary(http):
    await _create(http, MORTGAGE)
    await _create(http, CARD)
    s = (await http.get(f"{URL}/summary", headers=ADVISOR)).json()
    assert s["count"] == 2
    assert s["total_current_balance"] == "251500.50"
    assert s["count_by_type"] == {"mortgage": 1, "credit_card": 1}
    assert s["total_annual_payments"] == "24000.00"


async def test_client_isolation(http):
    created = await _create(http)
    other = "/api/v1/clients/c2/liabilities"
    assert (await http.get(f"{other}/{created['id']}", headers=ADVISOR)).status_code == 404
    assert (await http.get(other, headers=ADVISOR)).json()["total"] == 0
    assert (await http.delete(f"{other}/{created['id']}", headers=ADVISOR)).status_code == 404


@pytest.mark.parametrize(
    "headers", [{}, {"X-Caller-Type": "advisor"}, {"X-Caller-Type": "bogus", "X-User-Id": "x"}]
)
async def test_bad_caller_headers(http, headers):
    r = await http.post(URL, json=MORTGAGE, headers=headers)
    assert r.status_code == 422


@pytest.mark.parametrize(
    "patch",
    [
        {"current_balance": "-1"},
        {"account_last4": "12"},
        {"interest_rate": "150"},
        {"current_balance": None},
    ],
)
async def test_validation_errors(http, patch):
    r = await http.post(URL, json={**MORTGAGE, **patch}, headers=ADVISOR)
    assert r.status_code == 422


async def test_date_validation(http):
    bad = {**MORTGAGE, "origination_date": "2020-01-01", "maturity_date": "2019-01-01"}
    assert (await http.post(URL, json=bad, headers=ADVISOR)).status_code == 422
    created = await _create(http, {**MORTGAGE, "origination_date": "2020-01-01"})
    r = await http.patch(
        f"{URL}/{created['id']}", json={"maturity_date": "2019-01-01"}, headers=ADVISOR
    )
    assert r.status_code == 422


async def test_invalid_id_is_404(http):
    assert (await http.get(f"{URL}/not-an-id", headers=ADVISOR)).status_code == 404
