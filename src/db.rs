use mongodb::bson::{Document, doc};
use mongodb::options::ClientOptions;
use mongodb::{Client, Collection, Database, IndexModel};

use crate::error::AppError;

/// Handle to the `liabilities` collection. Cheap to clone.
#[derive(Clone)]
pub struct Store {
    pub(crate) db: Database,
    pub(crate) coll: Collection<Document>,
}

impl Store {
    pub async fn connect(uri: &str, db_name: &str) -> Result<Self, AppError> {
        let options = ClientOptions::parse(uri).await?;
        let client = Client::with_options(options)?;
        let db = client.database(db_name);
        let coll = db.collection::<Document>("liabilities");
        coll.create_indexes([
            IndexModel::builder().keys(doc! { "client_id": 1 }).build(),
            IndexModel::builder()
                .keys(doc! { "client_id": 1, "liability_type": 1 })
                .build(),
        ])
        .await?;
        Ok(Self { db, coll })
    }

    pub async fn ping(&self) -> Result<(), AppError> {
        self.db.run_command(doc! { "ping": 1 }).await?;
        Ok(())
    }

    /// Drop the whole database (tests).
    pub async fn drop_database(&self) -> Result<(), AppError> {
        self.db.drop().await?;
        Ok(())
    }
}
