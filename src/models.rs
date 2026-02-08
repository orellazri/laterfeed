use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, query_as, sqlite::SqlitePool};

#[derive(sqlx::Type, Serialize, Deserialize, Copy, Clone, Eq, PartialEq)]
#[repr(i64)]
pub enum EntrySourceType {
    Article = 0,
    Video = 1,
}

impl From<i64> for EntrySourceType {
    fn from(value: i64) -> Self {
        match value {
            0 => EntrySourceType::Article,
            1 => EntrySourceType::Video,
            _ => EntrySourceType::Article,
        }
    }
}

#[derive(FromRow, Serialize, Deserialize)]
pub struct Entry {
    pub id: i64,
    pub url: String,
    pub title: String,
    pub summary: Option<String>,
    pub source_type: EntrySourceType,
    pub created_at: DateTime<Utc>,
}

impl Entry {
    pub async fn create(
        pool: &SqlitePool,
        url: &str,
        title: &str,
        summary: Option<&str>,
        source_type: EntrySourceType,
    ) -> Result<Entry, sqlx::Error> {
        let now = Utc::now();

        query_as!(
            Entry,
            r#"
            INSERT INTO entries (url, title, summary, source_type, created_at) VALUES (?, ?, ?, ?, ?)
            RETURNING id, url, title, summary, source_type, created_at as "created_at: DateTime<Utc>"
            "#,
            url,
            title,
            summary,
            source_type,
            now
        )
        .fetch_one(pool)
        .await
    }

    pub async fn fetch_all(pool: &SqlitePool) -> Result<Vec<Entry>, sqlx::Error> {
        query_as!(
            Entry,
            r#"
            SELECT id, url, title, summary, source_type, created_at as "created_at: DateTime<Utc>"
            FROM entries ORDER BY created_at DESC
            "#
        )
        .fetch_all(pool)
        .await
    }
}
