#![allow(dead_code)]

use std::sync::OnceLock;
use std::sync::atomic::{AtomicU32, Ordering};

use axum::http::Method;
use axum_test::TestServer;
use liabilities::api::app;
use liabilities::db::Store;
use serde_json::Value;
use testcontainers_modules::mongo::Mongo;
use testcontainers_modules::testcontainers::ImageExt;
use testcontainers_modules::testcontainers::runners::AsyncRunner;

pub type Headers = [(&'static str, &'static str)];

pub const ADVISOR: [(&str, &str); 2] = [("X-Caller-Type", "advisor"), ("X-User-Id", "adv-1")];
pub const OTHER_ADVISOR: [(&str, &str); 2] = [("X-Caller-Type", "advisor"), ("X-User-Id", "adv-2")];
pub const CLIENT: [(&str, &str); 2] = [("X-Caller-Type", "client"), ("X-User-Id", "cli-1")];
pub const OTHER_CLIENT: [(&str, &str); 2] = [("X-Caller-Type", "client"), ("X-User-Id", "cli-2")];
pub const NO_HEADERS: [(&str, &str); 0] = [];

/// MongoDB URI: `MONGODB_URI` if set (e.g. the docker compose database), otherwise one shared
/// throwaway container per test binary. The container lives on its own runtime so it outlasts
/// the short-lived per-test runtimes.
fn mongo_uri() -> &'static str {
    static URI: OnceLock<String> = OnceLock::new();
    URI.get_or_init(|| {
        if let Ok(uri) = std::env::var("MONGODB_URI") {
            return uri;
        }
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("runtime");
            let container = rt
                .block_on(Mongo::default().with_tag("8").start())
                .expect("start MongoDB container (is Docker running?)");
            let port = rt
                .block_on(container.get_host_port_ipv4(27017))
                .expect("mongo port");
            tx.send(format!("mongodb://127.0.0.1:{port}"))
                .expect("send uri");
            loop {
                std::thread::park(); // keep the container (and runtime) alive until process exit
            }
        });
        rx.recv().expect("mongo uri")
    })
}

/// An in-process app on its own fresh database, dropped when the test ends.
pub struct Api {
    server: TestServer,
    store: Store,
}

impl Api {
    pub async fn new() -> Self {
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let uri = mongo_uri();
        let name = format!(
            "liabilities_test_{}_{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        );
        let store = Store::connect(uri, &name).await.expect("connect");
        Self {
            server: TestServer::new(app(store.clone())),
            store,
        }
    }

    pub async fn send(
        &self,
        method: Method,
        url: &str,
        headers: &Headers,
        body: Option<Value>,
        query: &[(&str, &str)],
    ) -> (u16, Value) {
        let mut req = self.server.method(method, url);
        for (k, v) in headers {
            req = req.add_header(*k, *v);
        }
        if !query.is_empty() {
            req = req.add_query_params(query);
        }
        if let Some(body) = body {
            req = req.json(&body);
        }
        let res = req.await;
        let status = res.status_code().as_u16();
        let text = res.text();
        let json = serde_json::from_str(&text).unwrap_or(Value::Null);
        (status, json)
    }

    pub async fn get(&self, url: &str, h: &Headers) -> (u16, Value) {
        self.send(Method::GET, url, h, None, &[]).await
    }
    pub async fn get_q(&self, url: &str, h: &Headers, q: &[(&str, &str)]) -> (u16, Value) {
        self.send(Method::GET, url, h, None, q).await
    }
    pub async fn post(&self, url: &str, h: &Headers, body: Value) -> (u16, Value) {
        self.send(Method::POST, url, h, Some(body), &[]).await
    }
    pub async fn patch(&self, url: &str, h: &Headers, body: Value) -> (u16, Value) {
        self.send(Method::PATCH, url, h, Some(body), &[]).await
    }
    pub async fn delete(&self, url: &str, h: &Headers) -> (u16, Value) {
        self.send(Method::DELETE, url, h, None, &[]).await
    }

    /// POST that must succeed with 201; returns the created liability.
    pub async fn create(&self, url: &str, h: &Headers, body: Value) -> Value {
        let (status, json) = self.post(url, h, body).await;
        assert_eq!(status, 201, "{json}");
        json
    }

    /// GET that must succeed with 200.
    pub async fn read(&self, url: &str, h: &Headers) -> Value {
        let (status, json) = self.get(url, h).await;
        assert_eq!(status, 200, "{json}");
        json
    }
}

impl Drop for Api {
    fn drop(&mut self) {
        let store = self.store.clone();
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            tokio::task::block_in_place(|| {
                let _ = handle.block_on(store.drop_database());
            });
        }
    }
}

impl Api {
    /// Load the sample scenarios into this test's database.
    pub async fn seed(&self) -> usize {
        liabilities::seed::seed(&self.store).await.expect("seed")
    }
}
