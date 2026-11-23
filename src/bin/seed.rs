//! Seed the dev database with sample client scenarios. Re-runnable: it first removes the
//! seeded clients' existing liabilities. Usage: `make seed` (or `cargo run --bin seed`).

use liabilities::config::Settings;
use liabilities::db::Store;
use liabilities::seed;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let settings = Settings::load()?;
    let store = Store::connect(&settings.mongodb_uri, &settings.mongodb_db).await?;
    let n = seed::seed(&store).await?;
    println!(
        "Seeded {n} liabilities for {} clients in '{}':",
        seed::scenarios().len(),
        settings.mongodb_db
    );
    for s in seed::scenarios() {
        let c = s.counts();
        println!(
            "  {:30} {} items, {} pending ({} edits, {} adds, {} deletes)  - {}",
            s.client_id,
            c.liabilities,
            c.edits + c.adds + c.deletes,
            c.edits,
            c.adds,
            c.deletes,
            s.title
        );
    }
    Ok(())
}
