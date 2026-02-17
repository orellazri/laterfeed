use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, query, query_as, sqlite::SqlitePool};

#[derive(sqlx::Type, Serialize, Deserialize, Copy, Clone, Debug, Eq, PartialEq)]
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
    pub body: Option<String>,
    pub source_type: EntrySourceType,
    pub created_at: DateTime<Utc>,
}

impl Entry {
    pub async fn create(
        pool: &SqlitePool,
        url: &str,
        title: &str,
        body: Option<&str>,
        source_type: EntrySourceType,
    ) -> Result<Entry, sqlx::Error> {
        let now = Utc::now();

        query_as!(
            Entry,
            r#"
            INSERT INTO entries (url, title, body, source_type, created_at) VALUES (?, ?, ?, ?, ?)
            RETURNING id, url, title, body, source_type, created_at as "created_at: DateTime<Utc>"
            "#,
            url,
            title,
            body,
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
            SELECT id, url, title, body, source_type, created_at as "created_at: DateTime<Utc>"
            FROM entries ORDER BY created_at DESC
            "#
        )
        .fetch_all(pool)
        .await
    }

    pub async fn fetch_latest(pool: &SqlitePool, limit: i64) -> Result<Vec<Entry>, sqlx::Error> {
        query_as!(
            Entry,
            r#"
            SELECT id, url, title, body, source_type, created_at as "created_at: DateTime<Utc>"
            FROM entries ORDER BY created_at DESC LIMIT ?
            "#,
            limit
        )
        .fetch_all(pool)
        .await
    }

    /// Delete an entry by ID. Returns true if an entry was deleted, false if not found.
    pub async fn delete_by_id(pool: &SqlitePool, id: i64) -> Result<bool, sqlx::Error> {
        let result = query!("DELETE FROM entries WHERE id = ?", id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    /// Delete entries older than the given cutoff datetime. Returns the number of entries deleted.
    pub async fn delete_older_than(
        pool: &SqlitePool,
        cutoff: DateTime<Utc>,
    ) -> Result<u64, sqlx::Error> {
        let result = query!("DELETE FROM entries WHERE created_at < ?", cutoff)
            .execute(pool)
            .await?;
        Ok(result.rows_affected())
    }

    /// Delete entries beyond the N most recent. Returns the number of entries deleted.
    pub async fn delete_beyond_limit(pool: &SqlitePool, max: u32) -> Result<u64, sqlx::Error> {
        let result = query!(
            r#"
            DELETE FROM entries WHERE id NOT IN (
                SELECT id FROM entries ORDER BY created_at DESC LIMIT ?
            )
            "#,
            max
        )
        .execute(pool)
        .await?;
        Ok(result.rows_affected())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entry_source_type_from_i64_article() {
        assert_eq!(EntrySourceType::from(0), EntrySourceType::Article);
    }

    #[test]
    fn entry_source_type_from_i64_video() {
        assert_eq!(EntrySourceType::from(1), EntrySourceType::Video);
    }

    #[test]
    fn entry_source_type_from_i64_unknown_defaults_to_article() {
        assert_eq!(EntrySourceType::from(2), EntrySourceType::Article);
        assert_eq!(EntrySourceType::from(-1), EntrySourceType::Article);
        assert_eq!(EntrySourceType::from(100), EntrySourceType::Article);
        assert_eq!(EntrySourceType::from(i64::MAX), EntrySourceType::Article);
        assert_eq!(EntrySourceType::from(i64::MIN), EntrySourceType::Article);
    }
}
