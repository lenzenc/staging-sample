mod common;

use common::{ADVISOR, Api, CLIENT, NO_HEADERS};
use liabilities::seed;
use serde_json::{Value, json};

#[tokio::test(flavor = "multi_thread")]
async fn scenarios_listed_without_headers() {
    let api = Api::new().await;
    let (status, body) = api.get("/api/v1/seed-scenarios", &NO_HEADERS).await;
    assert_eq!(status, 200);
    let ids: Vec<_> = body
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["client_id"].as_str().unwrap())
        .collect();
    let expected: Vec<_> = seed::scenarios()
        .iter()
        .map(|s| s.client_id.as_str())
        .collect();
    assert_eq!(ids, expected);
    assert_eq!(ids.len(), 11);
    assert!(
        body.as_array()
            .unwrap()
            .iter()
            .all(|s| s["title"] != "" && s["description"] != "")
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn scenario_counts() {
    let api = Api::new().await;
    let (_, body) = api.get("/api/v1/seed-scenarios", &NO_HEADERS).await;
    let by_id = |id: &str| -> Value {
        body.as_array()
            .unwrap()
            .iter()
            .find(|s| s["client_id"] == id)
            .unwrap()["counts"]
            .clone()
    };
    assert_eq!(
        by_id("client-family-homeowners"),
        json!({"liabilities": 5, "unchanged": 1, "edits": 2, "adds": 1, "deletes": 1})
    );
    assert_eq!(
        by_id("client-single-add"),
        json!({"liabilities": 1, "unchanged": 0, "edits": 0, "adds": 1, "deletes": 0})
    );
    assert_eq!(by_id("client-no-pending")["edits"], 0);
    assert_eq!(by_id("client-empty")["liabilities"], 0);
}

/// Seeding writes through the same codec the API reads with, so the views must match the
/// documented use cases.
#[tokio::test(flavor = "multi_thread")]
async fn seeded_data_reads_back_through_the_api() {
    let api = Api::new().await;
    let n = api.seed().await;
    assert_eq!(n, 33);

    let url = "/api/v1/clients/client-edits-only/liabilities";
    let adv = api.read(url, &ADVISOR).await;
    let cli = api.read(url, &CLIENT).await;
    let find = |v: &Value, name: &str| {
        v["items"]
            .as_array()
            .unwrap()
            .iter()
            .find(|i| i["name"] == name)
            .cloned()
            .unwrap()
    };

    let a = find(&adv, "Travel Amex");
    let c = find(&cli, "Travel Amex");
    assert_eq!(a["current_balance"], "4200.00");
    assert_eq!(
        a["pending_change"]["changes"]["current_balance"],
        json!({"current": "4200.00", "proposed": "3500.00"})
    );
    assert_eq!(c["current_balance"], "3500.00");
    assert!(c.get("pending_change").is_none());

    // pending adds only: advisor totals are 0, client totals are not
    let url = "/api/v1/clients/client-adds-only/liabilities/summary";
    assert_eq!(api.read(url, &ADVISOR).await["count"], 0);
    assert_eq!(api.read(url, &CLIENT).await["count"], 2);

    // explicit null staged by the client survives the round trip
    let url = "/api/v1/clients/client-small-business-owner/liabilities";
    let sba = find(&api.read(url, &ADVISOR).await, "SBA 7(a) loan");
    assert_eq!(
        sba["pending_change"]["changes"]["notes"],
        json!({"current": "Personally guaranteed.", "proposed": null})
    );
    assert_eq!(
        find(&api.read(url, &CLIENT).await, "SBA 7(a) loan")["notes"],
        Value::Null
    );

    // the empty client has no liabilities; re-seeding is idempotent
    assert_eq!(
        api.read("/api/v1/clients/client-empty/liabilities", &ADVISOR)
            .await["total"],
        0
    );
    assert_eq!(api.seed().await, 33);
    assert_eq!(
        api.read("/api/v1/clients/client-no-pending/liabilities", &ADVISOR)
            .await["total"],
        3
    );
}
