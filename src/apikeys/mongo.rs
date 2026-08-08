use crate::apikeys::model::ApiKeyDoc;
use futures_util::TryStreamExt;
use mongodb::{bson::doc, Client, Collection};

const COLLECTION: &str = "api_keys";

/// Read-only, thin wrapper
pub struct ApiKeyRepository {
    collection: Collection<ApiKeyDoc>,
}

impl ApiKeyRepository {
    pub fn new(client: &Client, db_name: &str) -> Self {
        Self { collection: client.database(db_name).collection(COLLECTION) }
    }

    pub async fn fetch_all(&self) -> mongodb::error::Result<Vec<ApiKeyDoc>> {
        let cursor = self.collection.find(doc! {}).await?;
        cursor.try_collect().await
    }
}