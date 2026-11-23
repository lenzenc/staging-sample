mod common;

use common::{ADVISOR, Api, NO_HEADERS, OTHER_ADVISOR};
use serde_json::{Value, json};

const URL: &str = "/api/v1/clients/c1/liabilities";

fn mortgage() -> Value {
    json!({
        "liability_type": "mortgage",
        "name": "Primary mortgage",
        "current_balance": "250000.00",
        "interest_rate": "6.5",
        "payment_amount": "2000.00",
        "payment_frequency": "monthly",
    })
}

fn card() -> Value {
    json!({"liability_type": "credit_card", "name": "Visa", "current_balance": "1500.50"})
}

fn with(base: Value, extra: Value) -> Value {
    let mut base = base;
    base.as_object_mut()
        .unwrap()
        .extend(extra.as_object().unwrap().clone());
    base
}

#[tokio::test(flavor = "multi_thread")]
async fn crud_and_audit() {
    let api = Api::new().await;
    let created = api.create(URL, &ADVISOR, mortgage()).await;
    assert_eq!(created["created_by"]["caller_type"], "advisor");
    let lid = created["id"].as_str().unwrap();

    let (status, body) = api
        .patch(
            &format!("{URL}/{lid}"),
            &OTHER_ADVISOR,
            json!({"current_balance": "240000.00"}),
        )
        .await;
    assert_eq!(status, 200);
    assert_eq!(body["current_balance"], "240000.00");
    assert_eq!(body["name"], "Primary mortgage"); // untouched
    assert_eq!(body["created_by"]["user_id"], "adv-1");
    assert_eq!(body["updated_by"]["user_id"], "adv-2");

    assert_eq!(
        api.read(&format!("{URL}/{lid}"), &ADVISOR).await["current_balance"],
        "240000.00"
    );
    assert_eq!(api.delete(&format!("{URL}/{lid}"), &ADVISOR).await.0, 204);
    assert_eq!(api.get(&format!("{URL}/{lid}"), &ADVISOR).await.0, 404);
}

#[tokio::test(flavor = "multi_thread")]
async fn list_filter_pagination() {
    let api = Api::new().await;
    api.create(URL, &ADVISOR, mortgage()).await;
    api.create(URL, &ADVISOR, card()).await;
    api.create(URL, &ADVISOR, card()).await;
    assert_eq!(api.read(URL, &ADVISOR).await["total"], 3);
    let (_, r) = api
        .get_q(URL, &ADVISOR, &[("liability_type", "credit_card")])
        .await;
    assert_eq!(r["total"], 2);
    let (_, r) = api
        .get_q(URL, &ADVISOR, &[("limit", "1"), ("skip", "1")])
        .await;
    assert_eq!(r["total"], 3);
    assert_eq!(r["items"].as_array().unwrap().len(), 1);
}

#[tokio::test(flavor = "multi_thread")]
async fn list_query_validation() {
    let api = Api::new().await;
    for q in [
        [("limit", "0")],
        [("limit", "201")],
        [("skip", "-1")],
        [("liability_type", "bogus")],
    ] {
        assert_eq!(api.get_q(URL, &ADVISOR, &q).await.0, 422, "{q:?}");
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn list_is_newest_first() {
    let api = Api::new().await;
    api.create(URL, &ADVISOR, with(card(), json!({"name": "first"})))
        .await;
    tokio::time::sleep(std::time::Duration::from_millis(5)).await;
    api.create(URL, &ADVISOR, with(card(), json!({"name": "second"})))
        .await;
    let names: Vec<_> = api.read(URL, &ADVISOR).await["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["name"].as_str().unwrap().to_owned())
        .collect();
    assert_eq!(names, ["second", "first"]);
}

#[tokio::test(flavor = "multi_thread")]
async fn summary() {
    let api = Api::new().await;
    api.create(URL, &ADVISOR, mortgage()).await;
    api.create(URL, &ADVISOR, card()).await;
    let s = api.read(&format!("{URL}/summary"), &ADVISOR).await;
    assert_eq!(s["count"], 2);
    assert_eq!(s["total_current_balance"], "251500.50");
    assert_eq!(s["count_by_type"], json!({"mortgage": 1, "credit_card": 1}));
    assert_eq!(s["total_annual_payments"], "24000.00");
}

#[tokio::test(flavor = "multi_thread")]
async fn client_isolation() {
    let api = Api::new().await;
    let created = api.create(URL, &ADVISOR, mortgage()).await;
    let id = created["id"].as_str().unwrap();
    let other = "/api/v1/clients/c2/liabilities";
    assert_eq!(api.get(&format!("{other}/{id}"), &ADVISOR).await.0, 404);
    assert_eq!(api.read(other, &ADVISOR).await["total"], 0);
    assert_eq!(api.delete(&format!("{other}/{id}"), &ADVISOR).await.0, 404);
}

#[tokio::test(flavor = "multi_thread")]
async fn bad_caller_headers() {
    let api = Api::new().await;
    let cases: [&common::Headers; 3] = [
        &NO_HEADERS,
        &[("X-Caller-Type", "advisor")],
        &[("X-Caller-Type", "bogus"), ("X-User-Id", "x")],
    ];
    for headers in cases {
        assert_eq!(
            api.post(URL, headers, mortgage()).await.0,
            422,
            "{headers:?}"
        );
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn validation_errors() {
    let api = Api::new().await;
    for bad in [
        json!({"current_balance": "-1"}),
        json!({"account_last4": "12"}),
        json!({"interest_rate": "150"}),
        json!({"current_balance": null}),
        json!({"current_balance": "1.234"}),
        json!({"name": ""}),
        json!({"liability_type": "bogus"}),
    ] {
        let (status, body) = api.post(URL, &ADVISOR, with(mortgage(), bad.clone())).await;
        assert_eq!(status, 422, "{bad}");
        assert!(body["detail"].is_array(), "{body}");
    }
    // malformed JSON is a 422 too
    let r = api
        .send(axum::http::Method::POST, URL, &ADVISOR, None, &[])
        .await;
    assert_eq!(r.0, 422);
}

#[tokio::test(flavor = "multi_thread")]
async fn numeric_money_is_accepted() {
    let api = Api::new().await;
    let created = api
        .create(
            URL,
            &ADVISOR,
            with(card(), json!({"current_balance": 12.5})),
        )
        .await;
    assert_eq!(created["current_balance"], "12.5");
}

#[tokio::test(flavor = "multi_thread")]
async fn date_validation() {
    let api = Api::new().await;
    let bad = with(
        mortgage(),
        json!({"origination_date": "2020-01-01", "maturity_date": "2019-01-01"}),
    );
    assert_eq!(api.post(URL, &ADVISOR, bad).await.0, 422);
    let created = api
        .create(
            URL,
            &ADVISOR,
            with(mortgage(), json!({"origination_date": "2020-01-01"})),
        )
        .await;
    let id = created["id"].as_str().unwrap();
    let (status, _) = api
        .patch(
            &format!("{URL}/{id}"),
            &ADVISOR,
            json!({"maturity_date": "2019-01-01"}),
        )
        .await;
    assert_eq!(status, 422);
    // dates round-trip as ISO strings
    let ok = api
        .patch(
            &format!("{URL}/{id}"),
            &ADVISOR,
            json!({"maturity_date": "2050-01-01"}),
        )
        .await;
    assert_eq!(ok.1["maturity_date"], "2050-01-01");
    assert_eq!(ok.1["origination_date"], "2020-01-01");
}

#[tokio::test(flavor = "multi_thread")]
async fn patch_rejects_null_on_required_and_bad_values() {
    let api = Api::new().await;
    let id = api.create(URL, &ADVISOR, mortgage()).await["id"]
        .as_str()
        .unwrap()
        .to_owned();
    for bad in [
        json!({"current_balance": null}),
        json!({"name": null}),
        json!({"account_last4": "x"}),
    ] {
        assert_eq!(
            api.patch(&format!("{URL}/{id}"), &ADVISOR, bad.clone())
                .await
                .0,
            422,
            "{bad}"
        );
    }
    // optional fields can be cleared
    let id2 = api
        .create(URL, &ADVISOR, with(card(), json!({"notes": "hello"})))
        .await["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let (status, body) = api
        .patch(&format!("{URL}/{id2}"), &ADVISOR, json!({"notes": null}))
        .await;
    assert_eq!(status, 200);
    assert_eq!(body["notes"], Value::Null);
}

#[tokio::test(flavor = "multi_thread")]
async fn invalid_id_is_404() {
    let api = Api::new().await;
    assert_eq!(api.get(&format!("{URL}/not-an-id"), &ADVISOR).await.0, 404);
}

#[tokio::test(flavor = "multi_thread")]
async fn health_and_harness() {
    let api = Api::new().await;
    let (status, body) = api.get("/health", &NO_HEADERS).await;
    assert_eq!((status, &body["status"]), (200, &json!("ok")));
    assert_eq!(
        api.send(axum::http::Method::GET, "/harness/", &NO_HEADERS, None, &[])
            .await
            .0,
        200
    );
}
