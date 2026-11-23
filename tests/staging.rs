mod common;

use common::{ADVISOR, Api, CLIENT, Headers, NO_HEADERS, OTHER_CLIENT};
use serde_json::{Value, json};

const URL: &str = "/api/v1/clients/c1/liabilities";

fn card() -> Value {
    json!({
        "liability_type": "credit_card",
        "name": "Visa",
        "current_balance": "1500.50",
        "payment_amount": "100.00",
        "payment_frequency": "monthly",
    })
}

fn card_with(extra: Value) -> Value {
    let mut c = card();
    c.as_object_mut()
        .unwrap()
        .extend(extra.as_object().unwrap().clone());
    c
}

async fn create_card(api: &Api) -> String {
    api.create(URL, &ADVISOR, card()).await["id"]
        .as_str()
        .unwrap()
        .to_owned()
}

async fn get(api: &Api, id: &str, h: &Headers) -> Value {
    api.read(&format!("{URL}/{id}"), h).await
}

async fn patch(api: &Api, id: &str, h: &Headers, body: Value) -> (u16, Value) {
    api.patch(&format!("{URL}/{id}"), h, body).await
}

#[tokio::test(flavor = "multi_thread")]
async fn client_patch_is_staged() {
    let api = Api::new().await;
    let id = create_card(&api).await;
    let (status, body) = patch(&api, &id, &CLIENT, json!({"current_balance": "900.00"})).await;
    assert_eq!(status, 200);
    assert_eq!(body["current_balance"], "900.00"); // client sees their own edit

    let adv = get(&api, &id, &ADVISOR).await;
    assert_eq!(adv["current_balance"], "1500.50"); // committed data unchanged
    assert_eq!(adv["updated_by"]["caller_type"], "advisor");
    let pc = &adv["pending_change"];
    assert_eq!(
        pc["changes"],
        json!({"current_balance": {"current": "1500.50", "proposed": "900.00"}})
    );
    assert_eq!(pc["created_by"]["user_id"], "cli-1");

    let cli = get(&api, &id, &CLIENT).await;
    assert_eq!(cli["current_balance"], "900.00");
    assert_eq!(cli["name"], "Visa"); // merged with non-staged data
    assert!(cli.get("pending_change").is_none()); // clients get only their updated view
}

#[tokio::test(flavor = "multi_thread")]
async fn no_pending_change_when_nothing_staged() {
    let api = Api::new().await;
    let id = create_card(&api).await;
    assert_eq!(
        get(&api, &id, &ADVISOR).await["pending_change"],
        Value::Null
    );
    assert!(
        get(&api, &id, &CLIENT)
            .await
            .get("pending_change")
            .is_none()
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn repeated_client_patches_merge() {
    let api = Api::new().await;
    let id = create_card(&api).await;
    patch(&api, &id, &CLIENT, json!({"current_balance": "900.00"})).await;
    patch(
        &api,
        &id,
        &OTHER_CLIENT,
        json!({"current_balance": "800.00", "notes": "paying down"}),
    )
    .await;
    let pc = get(&api, &id, &ADVISOR).await["pending_change"].clone();
    assert_eq!(pc["changes"]["current_balance"]["proposed"], "800.00");
    assert_eq!(
        pc["changes"]["notes"],
        json!({"current": null, "proposed": "paying down"})
    );
    assert_eq!(pc["created_by"]["user_id"], "cli-1");
    assert_eq!(pc["updated_by"]["user_id"], "cli-2");
}

#[tokio::test(flavor = "multi_thread")]
async fn client_can_stage_explicit_null() {
    let api = Api::new().await;
    let id = api
        .create(URL, &ADVISOR, card_with(json!({"notes": "old"})))
        .await["id"]
        .as_str()
        .unwrap()
        .to_owned();
    patch(&api, &id, &CLIENT, json!({"notes": null})).await;
    assert_eq!(get(&api, &id, &CLIENT).await["notes"], Value::Null);
    assert_eq!(get(&api, &id, &ADVISOR).await["notes"], "old");
}

#[tokio::test(flavor = "multi_thread")]
async fn reverting_to_committed_value_clears_staging() {
    let api = Api::new().await;
    let id = create_card(&api).await;
    patch(&api, &id, &CLIENT, json!({"current_balance": "900.00"})).await;
    patch(&api, &id, &CLIENT, json!({"current_balance": "1500.5"})).await;
    assert_eq!(
        get(&api, &id, &ADVISOR).await["pending_change"],
        Value::Null
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn client_patch_validates_merged_record() {
    let api = Api::new().await;
    let id = api
        .create(
            URL,
            &ADVISOR,
            card_with(json!({"origination_date": "2020-01-01"})),
        )
        .await["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let (status, _) = patch(&api, &id, &CLIENT, json!({"maturity_date": "2019-01-01"})).await;
    assert_eq!(status, 422);
    assert_eq!(
        get(&api, &id, &ADVISOR).await["pending_change"],
        Value::Null
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn advisor_patch_applies_and_drops_overlapping_staged_fields() {
    let api = Api::new().await;
    let id = create_card(&api).await;
    patch(
        &api,
        &id,
        &CLIENT,
        json!({"current_balance": "900.00", "notes": "hi"}),
    )
    .await;
    let (status, body) = patch(&api, &id, &ADVISOR, json!({"current_balance": "1000.00"})).await;
    assert_eq!(status, 200);
    assert_eq!(body["current_balance"], "1000.00");
    assert_eq!(body["updated_by"]["caller_type"], "advisor");
    let pc = &body["pending_change"];
    let keys: Vec<_> = pc["changes"].as_object().unwrap().keys().collect();
    assert_eq!(keys, ["notes"]); // overlap dropped, the rest kept
    assert_eq!(pc["updated_by"]["user_id"], "cli-1"); // prune keeps client stamps
    assert_eq!(get(&api, &id, &CLIENT).await["current_balance"], "1000.00");

    patch(&api, &id, &ADVISOR, json!({"notes": "advisor note"})).await;
    assert_eq!(
        get(&api, &id, &ADVISOR).await["pending_change"],
        Value::Null
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn client_post_is_a_pending_create() {
    let api = Api::new().await;
    create_card(&api).await; // committed, 1500.50
    let new = api
        .create(
            URL,
            &CLIENT,
            card_with(json!({"name": "New card", "current_balance": "500.00"})),
        )
        .await;
    assert!(new.get("pending_change").is_none()); // the client just sees their new record
    let adv = get(&api, new["id"].as_str().unwrap(), &ADVISOR).await;
    assert_eq!(adv["pending_change"]["action"], "create");
    assert_eq!(adv["name"], "New card");
    assert_eq!(api.read(URL, &ADVISOR).await["total"], 2);

    // Not committed yet: the advisor's totals leave it out, the client's include it.
    let a = api.read(&format!("{URL}/summary"), &ADVISOR).await;
    let c = api.read(&format!("{URL}/summary"), &CLIENT).await;
    assert_eq!(
        (&a["total_current_balance"], &a["count"]),
        (&json!("1500.50"), &json!(1))
    );
    assert_eq!(
        (&c["total_current_balance"], &c["count"]),
        (&json!("2000.50"), &json!(2))
    );
    assert_eq!(a["pending_by_action"], json!({"create": 1}));
    assert_eq!(c["pending_by_action"], json!({}));
    assert_eq!(c["pending_change_count"], 0);
}

#[tokio::test(flavor = "multi_thread")]
async fn client_patch_on_pending_create_edits_record() {
    let api = Api::new().await;
    let new = api.create(URL, &CLIENT, card()).await;
    let id = new["id"].as_str().unwrap();
    let (status, _) = patch(&api, id, &CLIENT, json!({"current_balance": "42.00"})).await;
    assert_eq!(status, 200);
    let adv = get(&api, id, &ADVISOR).await;
    assert_eq!(adv["current_balance"], "42.00"); // the record itself is the proposal
    assert_eq!(adv["pending_change"]["action"], "create");
    assert_eq!(adv["pending_change"]["changes"], json!({}));
}

#[tokio::test(flavor = "multi_thread")]
async fn client_delete_of_pending_create_removes_it() {
    let api = Api::new().await;
    let new = api.create(URL, &CLIENT, card()).await;
    let id = new["id"].as_str().unwrap();
    assert_eq!(api.delete(&format!("{URL}/{id}"), &CLIENT).await.0, 204);
    assert_eq!(api.get(&format!("{URL}/{id}"), &ADVISOR).await.0, 404);
}

#[tokio::test(flavor = "multi_thread")]
async fn client_delete_is_staged_and_hidden_from_client() {
    let api = Api::new().await;
    let id = create_card(&api).await;
    patch(&api, &id, &CLIENT, json!({"notes": "x"})).await; // earlier staged edit
    assert_eq!(api.delete(&format!("{URL}/{id}"), &CLIENT).await.0, 204);

    // Gone for the client everywhere.
    assert_eq!(api.get(&format!("{URL}/{id}"), &CLIENT).await.0, 404);
    assert_eq!(
        patch(&api, &id, &CLIENT, json!({"notes": "y"})).await.0,
        404
    );
    assert_eq!(api.delete(&format!("{URL}/{id}"), &CLIENT).await.0, 404);
    assert_eq!(api.read(URL, &CLIENT).await["total"], 0);
    let c = api.read(&format!("{URL}/summary"), &CLIENT).await;
    assert_eq!(
        (&c["count"], &c["total_current_balance"]),
        (&json!(0), &json!("0"))
    );

    // The advisor still sees the committed record, flagged; staged edits were discarded.
    let adv = get(&api, &id, &ADVISOR).await;
    assert_eq!(adv["current_balance"], "1500.50");
    assert_eq!(adv["notes"], Value::Null);
    assert_eq!(adv["pending_change"]["action"], "delete");
    assert_eq!(adv["pending_change"]["changes"], json!({}));
    let a = api.read(&format!("{URL}/summary"), &ADVISOR).await;
    assert_eq!(a["count"], 1);
    assert_eq!(a["pending_by_action"], json!({"delete": 1}));
}

#[tokio::test(flavor = "multi_thread")]
async fn advisor_delete_hard_deletes_pending_delete() {
    let api = Api::new().await;
    let id = create_card(&api).await;
    api.delete(&format!("{URL}/{id}"), &CLIENT).await;
    assert_eq!(api.delete(&format!("{URL}/{id}"), &ADVISOR).await.0, 204);
    assert_eq!(api.get(&format!("{URL}/{id}"), &ADVISOR).await.0, 404);
}

#[tokio::test(flavor = "multi_thread")]
async fn advisor_patch_keeps_create_and_delete_flags() {
    let api = Api::new().await;
    let created = api.create(URL, &CLIENT, card()).await;
    let (_, r) = patch(
        &api,
        created["id"].as_str().unwrap(),
        &ADVISOR,
        json!({"name": "Fixed"}),
    )
    .await;
    assert_eq!(r["name"], "Fixed");
    assert_eq!(r["pending_change"]["action"], "create");

    let id = create_card(&api).await;
    api.delete(&format!("{URL}/{id}"), &CLIENT).await;
    let (_, r) = patch(&api, &id, &ADVISOR, json!({"name": "Fixed"})).await;
    assert_eq!(r["name"], "Fixed");
    assert_eq!(r["pending_change"]["action"], "delete");
}

#[tokio::test(flavor = "multi_thread")]
async fn delete_requires_caller_headers() {
    let api = Api::new().await;
    let id = create_card(&api).await;
    assert_eq!(api.delete(&format!("{URL}/{id}"), &NO_HEADERS).await.0, 422);
}

#[tokio::test(flavor = "multi_thread")]
async fn list_filter_uses_view_of_caller() {
    let api = Api::new().await;
    let id = create_card(&api).await;
    patch(
        &api,
        &id,
        &CLIENT,
        json!({"liability_type": "personal_loan"}),
    )
    .await;
    let q = [("liability_type", "personal_loan")];
    let (_, cli) = api.get_q(URL, &CLIENT, &q).await;
    let (_, adv) = api.get_q(URL, &ADVISOR, &q).await;
    assert_eq!(cli["total"], 1);
    assert_eq!(cli["items"][0]["liability_type"], "personal_loan");
    assert_eq!(adv["total"], 0);
    let q = [("liability_type", "credit_card")];
    assert_eq!(api.get_q(URL, &CLIENT, &q).await.1["total"], 0);
    assert_eq!(api.get_q(URL, &ADVISOR, &q).await.1["total"], 1);
}

#[tokio::test(flavor = "multi_thread")]
async fn list_shows_pending_by_caller() {
    let api = Api::new().await;
    let id = create_card(&api).await;
    patch(&api, &id, &CLIENT, json!({"current_balance": "900.00"})).await;
    let adv = api.read(URL, &ADVISOR).await["items"][0].clone();
    let cli = api.read(URL, &CLIENT).await["items"][0].clone();
    assert_eq!(adv["current_balance"], "1500.50");
    assert!(adv["pending_change"].is_object());
    assert_eq!(cli["current_balance"], "900.00");
    assert!(cli.get("pending_change").is_none());
}

#[tokio::test(flavor = "multi_thread")]
async fn summary_differs_by_caller() {
    let api = Api::new().await;
    let id = create_card(&api).await;
    api.create(
        URL,
        &ADVISOR,
        card_with(json!({"name": "MC", "current_balance": "500.00"})),
    )
    .await;
    patch(&api, &id, &CLIENT, json!({"current_balance": "900.00"})).await;
    let adv = api.read(&format!("{URL}/summary"), &ADVISOR).await;
    let cli = api.read(&format!("{URL}/summary"), &CLIENT).await;
    assert_eq!(adv["total_current_balance"], "2000.50");
    assert_eq!(adv["pending_change_count"], 1);
    assert_eq!(cli["total_current_balance"], "1400.00");
    assert_eq!(cli["pending_change_count"], 0);
}

#[tokio::test(flavor = "multi_thread")]
async fn reads_require_caller_headers() {
    let api = Api::new().await;
    let id = create_card(&api).await;
    assert_eq!(api.get(URL, &NO_HEADERS).await.0, 422);
    assert_eq!(api.get(&format!("{URL}/{id}"), &NO_HEADERS).await.0, 422);
    assert_eq!(api.get(&format!("{URL}/summary"), &NO_HEADERS).await.0, 422);
}

#[tokio::test(flavor = "multi_thread")]
async fn client_view_has_no_change_info() {
    let api = Api::new().await;
    let edited = api
        .create(URL, &ADVISOR, card_with(json!({"name": "Edited"})))
        .await["id"]
        .as_str()
        .unwrap()
        .to_owned();
    api.create(URL, &ADVISOR, card_with(json!({"name": "Untouched"})))
        .await;
    let gone = api
        .create(URL, &ADVISOR, card_with(json!({"name": "Gone"})))
        .await["id"]
        .as_str()
        .unwrap()
        .to_owned();
    patch(&api, &edited, &CLIENT, json!({"current_balance": "10.00"})).await;
    api.create(URL, &CLIENT, card_with(json!({"name": "Added"})))
        .await;
    api.delete(&format!("{URL}/{gone}"), &CLIENT).await;

    let cli = api.read(URL, &CLIENT).await;
    assert_eq!(cli["total"], 3);
    let items = cli["items"].as_array().unwrap();
    let by_name = |n: &str| items.iter().find(|i| i["name"] == n).cloned();
    let names: std::collections::BTreeSet<_> =
        items.iter().map(|i| i["name"].as_str().unwrap()).collect();
    assert_eq!(names, ["Added", "Edited", "Untouched"].into()); // delete hidden, add shown
    assert_eq!(by_name("Edited").unwrap()["current_balance"], "10.00"); // staged value wins
    assert!(items.iter().all(|i| i.get("pending_change").is_none()));
    let s = api.read(&format!("{URL}/summary"), &CLIENT).await;
    assert_eq!(s["pending_by_action"], json!({}));

    let adv = api.read(URL, &ADVISOR).await;
    assert_eq!(adv["total"], 4);
    // the key is present on every advisor item (null when nothing is staged)
    assert!(
        adv["items"]
            .as_array()
            .unwrap()
            .iter()
            .all(|i| i.get("pending_change").is_some())
    );
}
