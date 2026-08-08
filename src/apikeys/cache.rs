use crate::apikeys::model::KeyRecord;
use crate::apikeys::mongo::ApiKeyRepository;
use dashmap::DashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

/// In-memory lookup table, keyed by key_hash.
pub struct ApiKeyCache {
    entries: DashMap<String, KeyRecord>,
    last_manual_refresh: AtomicU64,
}

impl ApiKeyCache {
    pub fn empty() -> Self {
        Self { entries: DashMap::new(), last_manual_refresh: AtomicU64::new(0) }
    }

    pub fn get(&self, key_hash: &str) -> Option<KeyRecord> {
        self.entries.get(key_hash).map(|e| e.value().clone())
    }

    /// Full replace from a fresh Mongo snapshot
    pub async fn refresh(&self, repo: &ApiKeyRepository) -> anyhow::Result<()> {
        let docs = repo.fetch_all().await?;
        let fresh: DashMap<String, KeyRecord> = DashMap::with_capacity(docs.len());
        for doc in docs {
            fresh.insert(doc.key_hash.clone(), KeyRecord::from(doc));
        }
        self.entries.clear();
        for (key, value) in fresh {
            self.entries.insert(key, value);
        }
        Ok(())
    }


    pub async fn refresh_debounced(&self, repo: &ApiKeyRepository, min_gap_secs: u64) -> anyhow::Result<bool> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
        let last = self.last_manual_refresh.load(Ordering::Acquire);
        if now.saturating_sub(last) < min_gap_secs {
            return Ok(false);
        }
        self.last_manual_refresh.store(now, Ordering::Release);
        self.refresh(repo).await?;
        Ok(true)
    }
}

pub type SharedApiKeyCache = Arc<ApiKeyCache>;