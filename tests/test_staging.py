from tests.conftest import ADVISOR, CLIENT

URL = "/api/v1/clients/c1/liabilities"
CARD = {
    "liability_type": "credit_card",
    "name": "Visa",
    "current_balance": "1500.50",
    "payment_amount": "100.00",
    "payment_frequency": "monthly",
}


async def _create(http, body=CARD, headers=ADVISOR):
    r = await http.post(URL, json=body, headers=headers)
    assert r.status_code == 201, r.text
    return r.json()


async def _get(http, lid, headers):
    r = await http.get(f"{URL}/{lid}", headers=headers)
    assert r.status_code == 200, r.text
    return r.json()


async def test_client_patch_is_staged(http):
    lid = (await _create(http))["id"]
    r = await http.patch(f"{URL}/{lid}", json={"current_balance": "900.00"}, headers=CLIENT)
    assert r.status_code == 200
    assert r.json()["current_balance"] == "900.00"  # client sees their own edit

    adv = await _get(http, lid, ADVISOR)
    assert adv["current_balance"] == "1500.50"  # committed data unchanged
    assert adv["updated_by"]["caller_type"] == "advisor"
    pc = adv["pending_change"]
    assert pc["changes"] == {"current_balance": {"current": "1500.50", "proposed": "900.00"}}
    assert pc["created_by"]["user_id"] == "cli-1"

    cli = await _get(http, lid, CLIENT)
    assert cli["current_balance"] == "900.00"
    assert cli["name"] == "Visa"  # merged with non-staged data
    assert "pending_change" not in cli  # clients get only their updated view


async def test_no_pending_change_when_nothing_staged(http):
    lid = (await _create(http))["id"]
    assert (await _get(http, lid, ADVISOR))["pending_change"] is None
    assert "pending_change" not in await _get(http, lid, CLIENT)


async def test_repeated_client_patches_merge(http):
    lid = (await _create(http))["id"]
    await http.patch(f"{URL}/{lid}", json={"current_balance": "900.00"}, headers=CLIENT)
    other = {**CLIENT, "X-User-Id": "cli-2"}
    await http.patch(
        f"{URL}/{lid}", json={"current_balance": "800.00", "notes": "paying down"}, headers=other
    )
    pc = (await _get(http, lid, ADVISOR))["pending_change"]
    assert pc["changes"]["current_balance"]["proposed"] == "800.00"
    assert pc["changes"]["notes"] == {"current": None, "proposed": "paying down"}
    assert pc["created_by"]["user_id"] == "cli-1"
    assert pc["updated_by"]["user_id"] == "cli-2"


async def test_client_can_stage_explicit_null(http):
    lid = (await _create(http, {**CARD, "notes": "old"}))["id"]
    await http.patch(f"{URL}/{lid}", json={"notes": None}, headers=CLIENT)
    assert (await _get(http, lid, CLIENT))["notes"] is None
    assert (await _get(http, lid, ADVISOR))["notes"] == "old"


async def test_reverting_to_committed_value_clears_staging(http):
    lid = (await _create(http))["id"]
    await http.patch(f"{URL}/{lid}", json={"current_balance": "900.00"}, headers=CLIENT)
    await http.patch(f"{URL}/{lid}", json={"current_balance": "1500.5"}, headers=CLIENT)
    assert (await _get(http, lid, ADVISOR))["pending_change"] is None


async def test_client_patch_validates_merged_record(http):
    lid = (await _create(http, {**CARD, "origination_date": "2020-01-01"}))["id"]
    r = await http.patch(f"{URL}/{lid}", json={"maturity_date": "2019-01-01"}, headers=CLIENT)
    assert r.status_code == 422
    assert (await _get(http, lid, ADVISOR))["pending_change"] is None


async def test_advisor_patch_applies_and_drops_overlapping_staged_fields(http):
    lid = (await _create(http))["id"]
    await http.patch(
        f"{URL}/{lid}", json={"current_balance": "900.00", "notes": "hi"}, headers=CLIENT
    )
    r = await http.patch(f"{URL}/{lid}", json={"current_balance": "1000.00"}, headers=ADVISOR)
    assert r.status_code == 200
    assert r.json()["current_balance"] == "1000.00"
    assert r.json()["updated_by"]["caller_type"] == "advisor"
    pc = r.json()["pending_change"]
    assert list(pc["changes"]) == ["notes"]  # overlap dropped, the rest kept
    assert pc["updated_by"]["user_id"] == "cli-1"  # prune keeps client stamps
    assert (await _get(http, lid, CLIENT))["current_balance"] == "1000.00"

    await http.patch(f"{URL}/{lid}", json={"notes": "advisor note"}, headers=ADVISOR)
    assert (await _get(http, lid, ADVISOR))["pending_change"] is None


async def test_client_post_is_a_pending_create(http):
    await _create(http)  # committed, 1500.50
    new = await _create(http, {**CARD, "name": "New card", "current_balance": "500.00"}, CLIENT)
    assert "pending_change" not in new  # the client just sees their new record
    adv = await _get(http, new["id"], ADVISOR)
    assert adv["pending_change"]["action"] == "create" and adv["name"] == "New card"
    assert (await http.get(URL, headers=ADVISOR)).json()["total"] == 2
    # Not committed yet: the advisor's totals leave it out, the client's include it.
    a = (await http.get(f"{URL}/summary", headers=ADVISOR)).json()
    c = (await http.get(f"{URL}/summary", headers=CLIENT)).json()
    assert a["total_current_balance"] == "1500.50" and a["count"] == 1
    assert c["total_current_balance"] == "2000.50" and c["count"] == 2
    assert a["pending_by_action"] == {"create": 1}
    assert c["pending_by_action"] == {} and c["pending_change_count"] == 0


async def test_client_patch_on_pending_create_edits_record(http):
    new = await _create(http, headers=CLIENT)
    r = await http.patch(f"{URL}/{new['id']}", json={"current_balance": "42.00"}, headers=CLIENT)
    assert r.status_code == 200
    adv = await _get(http, new["id"], ADVISOR)
    assert adv["current_balance"] == "42.00"  # the record itself is the proposal
    assert adv["pending_change"]["action"] == "create" and adv["pending_change"]["changes"] == {}


async def test_client_delete_of_pending_create_removes_it(http):
    new = await _create(http, headers=CLIENT)
    assert (await http.delete(f"{URL}/{new['id']}", headers=CLIENT)).status_code == 204
    assert (await http.get(f"{URL}/{new['id']}", headers=ADVISOR)).status_code == 404


async def test_client_delete_is_staged_and_hidden_from_client(http):
    lid = (await _create(http))["id"]
    await http.patch(f"{URL}/{lid}", json={"notes": "x"}, headers=CLIENT)  # earlier staged edit
    assert (await http.delete(f"{URL}/{lid}", headers=CLIENT)).status_code == 204

    # Gone for the client everywhere.
    assert (await http.get(f"{URL}/{lid}", headers=CLIENT)).status_code == 404
    assert (
        await http.patch(f"{URL}/{lid}", json={"notes": "y"}, headers=CLIENT)
    ).status_code == 404
    assert (await http.delete(f"{URL}/{lid}", headers=CLIENT)).status_code == 404
    assert (await http.get(URL, headers=CLIENT)).json()["total"] == 0
    c = (await http.get(f"{URL}/summary", headers=CLIENT)).json()
    assert c["count"] == 0 and c["total_current_balance"] == "0"

    # The advisor still sees the committed record, flagged; staged edits were discarded.
    adv = await _get(http, lid, ADVISOR)
    assert adv["current_balance"] == "1500.50" and adv["notes"] is None
    assert adv["pending_change"]["action"] == "delete" and adv["pending_change"]["changes"] == {}
    a = (await http.get(f"{URL}/summary", headers=ADVISOR)).json()
    assert a["count"] == 1 and a["pending_by_action"] == {"delete": 1}


async def test_advisor_delete_hard_deletes_pending_delete(http):
    lid = (await _create(http))["id"]
    await http.delete(f"{URL}/{lid}", headers=CLIENT)
    assert (await http.delete(f"{URL}/{lid}", headers=ADVISOR)).status_code == 204
    assert (await http.get(f"{URL}/{lid}", headers=ADVISOR)).status_code == 404


async def test_advisor_patch_keeps_create_and_delete_flags(http):
    created = await _create(http, headers=CLIENT)
    r = await http.patch(f"{URL}/{created['id']}", json={"name": "Fixed"}, headers=ADVISOR)
    assert r.json()["name"] == "Fixed" and r.json()["pending_change"]["action"] == "create"
    lid = (await _create(http))["id"]
    await http.delete(f"{URL}/{lid}", headers=CLIENT)
    r = await http.patch(f"{URL}/{lid}", json={"name": "Fixed"}, headers=ADVISOR)
    assert r.json()["name"] == "Fixed" and r.json()["pending_change"]["action"] == "delete"


async def test_delete_requires_caller_headers(http):
    lid = (await _create(http))["id"]
    assert (await http.delete(f"{URL}/{lid}")).status_code == 422


async def test_list_filter_uses_view_of_caller(http):
    lid = (await _create(http))["id"]
    await http.patch(f"{URL}/{lid}", json={"liability_type": "personal_loan"}, headers=CLIENT)
    params = {"liability_type": "personal_loan"}
    cli = (await http.get(URL, params=params, headers=CLIENT)).json()
    adv = (await http.get(URL, params=params, headers=ADVISOR)).json()
    assert cli["total"] == 1 and cli["items"][0]["liability_type"] == "personal_loan"
    assert adv["total"] == 0
    params = {"liability_type": "credit_card"}
    assert (await http.get(URL, params=params, headers=CLIENT)).json()["total"] == 0
    assert (await http.get(URL, params=params, headers=ADVISOR)).json()["total"] == 1


async def test_list_shows_pending_by_caller(http):
    lid = (await _create(http))["id"]
    await http.patch(f"{URL}/{lid}", json={"current_balance": "900.00"}, headers=CLIENT)
    adv = (await http.get(URL, headers=ADVISOR)).json()["items"][0]
    cli = (await http.get(URL, headers=CLIENT)).json()["items"][0]
    assert adv["current_balance"] == "1500.50" and adv["pending_change"]
    assert cli["current_balance"] == "900.00" and "pending_change" not in cli


async def test_summary_differs_by_caller(http):
    lid = (await _create(http))["id"]
    await _create(http, {**CARD, "name": "MC", "current_balance": "500.00"})
    await http.patch(f"{URL}/{lid}", json={"current_balance": "900.00"}, headers=CLIENT)
    adv = (await http.get(f"{URL}/summary", headers=ADVISOR)).json()
    cli = (await http.get(f"{URL}/summary", headers=CLIENT)).json()
    assert adv["total_current_balance"] == "2000.50" and adv["pending_change_count"] == 1
    assert cli["total_current_balance"] == "1400.00" and cli["pending_change_count"] == 0


async def test_reads_require_caller_headers(http):
    lid = (await _create(http))["id"]
    assert (await http.get(URL)).status_code == 422
    assert (await http.get(f"{URL}/{lid}")).status_code == 422
    assert (await http.get(f"{URL}/summary")).status_code == 422


async def test_client_view_has_no_change_info(http):
    edited = (await _create(http, {**CARD, "name": "Edited"}))["id"]
    await _create(http, {**CARD, "name": "Untouched"})
    gone = (await _create(http, {**CARD, "name": "Gone"}))["id"]
    await http.patch(f"{URL}/{edited}", json={"current_balance": "10.00"}, headers=CLIENT)
    await _create(http, {**CARD, "name": "Added"}, CLIENT)
    await http.delete(f"{URL}/{gone}", headers=CLIENT)

    cli = (await http.get(URL, headers=CLIENT)).json()
    assert cli["total"] == 3
    by_name = {i["name"]: i for i in cli["items"]}
    assert set(by_name) == {"Edited", "Untouched", "Added"}  # delete hidden, add shown
    assert by_name["Edited"]["current_balance"] == "10.00"  # staged value wins
    assert all("pending_change" not in i for i in cli["items"])
    assert (await http.get(f"{URL}/summary", headers=CLIENT)).json()["pending_by_action"] == {}

    adv = (await http.get(URL, headers=ADVISOR)).json()
    assert adv["total"] == 4
    assert all("pending_change" in i for i in adv["items"])  # null when nothing staged
