use liabilities::api::app;
use liabilities::config::Settings;
use liabilities::db::Store;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "liabilities=info,tower_http=info".into()),
        )
        .init();

    let settings = Settings::load()?;
    let store = Store::connect(&settings.mongodb_uri, &settings.mongodb_db).await?;
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", settings.port)).await?;
    tracing::info!(env = %settings.app_env, "listening on http://localhost:{}", settings.port);
    axum::serve(listener, app(store))
        .with_graceful_shutdown(shutdown())
        .await?;
    Ok(())
}

async fn shutdown() {
    let _ = tokio::signal::ctrl_c().await;
}
