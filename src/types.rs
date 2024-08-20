use chrono::{DateTime, Utc};

/// Defines the content sent over the PubSub protocol.
pub struct Content {
    pub sender: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
}