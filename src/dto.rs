use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::models;

#[derive(Serialize, Deserialize, ToSchema, Clone)]
#[serde(rename_all = "lowercase")]
pub enum EntrySourceType {
    Article,
    Video,
}

impl From<models::EntrySourceType> for EntrySourceType {
    fn from(source_type: models::EntrySourceType) -> Self {
        match source_type {
            models::EntrySourceType::Article => EntrySourceType::Article,
            models::EntrySourceType::Video => EntrySourceType::Video,
        }
    }
}

impl From<EntrySourceType> for models::EntrySourceType {
    fn from(source_type: EntrySourceType) -> Self {
        match source_type {
            EntrySourceType::Article => models::EntrySourceType::Article,
            EntrySourceType::Video => models::EntrySourceType::Video,
        }
    }
}

#[derive(Serialize, Deserialize, ToSchema, Validate)]
pub struct AddEntryRequest {
    #[validate(url)]
    pub url: String,
    pub title: Option<String>,
    pub source_type: EntrySourceType,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct EntryResponse {
    pub id: i64,
    pub url: String,
    pub title: String,
    pub body: Option<String>,
    pub source_type: EntrySourceType,
    pub created_at: DateTime<Utc>,
}

impl From<models::Entry> for EntryResponse {
    fn from(entry: models::Entry) -> Self {
        Self {
            id: entry.id,
            url: entry.url,
            title: entry.title,
            body: entry.body,
            source_type: entry.source_type.into(),
            created_at: entry.created_at,
        }
    }
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ListEntriesResponse {
    pub entries: Vec<EntryResponse>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn entry_response_from_model_entry() {
        let now = Utc::now();
        let entry = models::Entry {
            id: 42,
            url: "https://example.com".to_string(),
            title: "Test Title".to_string(),
            body: Some("Test Body".to_string()),
            source_type: models::EntrySourceType::Video,
            created_at: now,
        };

        let response: EntryResponse = entry.into();

        assert_eq!(response.id, 42);
        assert_eq!(response.url, "https://example.com");
        assert_eq!(response.title, "Test Title");
        assert_eq!(response.body, Some("Test Body".to_string()));
        assert!(matches!(response.source_type, EntrySourceType::Video));
        assert_eq!(response.created_at, now);
    }
}
