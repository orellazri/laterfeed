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
    pub url: String,
    pub title: Option<String>,
    pub summary: Option<String>,
    pub source_type: Option<EntrySourceType>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct EntryResponse {
    pub id: i64,
    pub url: String,
    pub title: String,
    pub summary: Option<String>,
    pub source_type: EntrySourceType,
    pub created_at: DateTime<Utc>,
}

impl From<models::Entry> for EntryResponse {
    fn from(entry: models::Entry) -> Self {
        Self {
            id: entry.id,
            url: entry.url,
            title: entry.title,
            summary: entry.summary,
            source_type: entry.source_type.into(),
            created_at: entry.created_at,
        }
    }
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ListEntriesResponse {
    pub entries: Vec<EntryResponse>,
}
