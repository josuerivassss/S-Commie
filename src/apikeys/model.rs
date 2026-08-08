use chrono::{DateTime, Utc};
use mongodb::bson::DateTime as BsonDateTime;
use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Plan {
    Basic,
    Pro,
    Partner,
    Test,
}

impl Plan {
    /// Requests allowed per rolling 24h window. None = unlimited (Partner).
    pub fn daily_quota(&self) -> Option<u32> {
        match self {
            Plan::Basic => Some(50),
            Plan::Pro => Some(1500),
            Plan::Partner => None,
            Plan::Test => Some(5),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Plan::Basic => "basic",
            Plan::Pro => "pro",
            Plan::Partner => "partner",
            Plan::Test => "test",
        }
    }

    fn from_str(value: &str) -> Self {
        match value {
            "pro" => Plan::Pro,
            "partner" => Plan::Partner,
            "test" => Plan::Test,
            _ => Plan::Basic, // unknown/malformed value fails safe to the most restrictive tier
        }
    }
}


#[derive(Debug, Deserialize)]
pub struct ApiKeyDoc {
    pub key_hash: String,
    pub discord_id: String,
    pub plan: String,
    pub banned: bool,
    pub expires_at: Option<BsonDateTime>,
}

#[derive(Debug, Clone)]
pub struct KeyRecord {
    pub discord_id: String,
    pub plan: Plan,
    pub banned: bool,
    pub expires_at: Option<DateTime<Utc>>,
}

fn bson_to_chrono(value: BsonDateTime) -> Option<DateTime<Utc>> {
    DateTime::from_timestamp_millis(value.timestamp_millis())
}

impl From<ApiKeyDoc> for KeyRecord {
    fn from(doc: ApiKeyDoc) -> Self {
        Self {
            discord_id: doc.discord_id,
            plan: Plan::from_str(&doc.plan),
            banned: doc.banned,
            expires_at: doc.expires_at.and_then(bson_to_chrono),
        }
    }
}

impl KeyRecord {
    /// Pro decays to Basic once expired -- computed on read, never written
    /// back (renewal/decay bookkeeping is the bot's job).
    pub fn effective_plan(&self) -> Plan {
        match (self.plan, self.expires_at) {
            (Plan::Pro, Some(expires_at)) if expires_at <= Utc::now() => Plan::Basic,
            (plan, _) => plan,
        }
    }
}