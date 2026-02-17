use std::time::Duration;

use chrono::Utc;
use sqlx::SqlitePool;
use tracing::{error, info};

const CLEANUP_INTERVAL_SECS: u64 = 43_200; // 12 hours

/// Starts the background cleanup task if retention settings are configured.
///
/// - Deletes entries older than `retention_days` (if set and > 0)
/// - Deletes entries beyond `max_entries` count (if set and > 0)
pub fn start_cleanup_task(pool: SqlitePool, retention_days: Option<u32>, max_entries: Option<u32>) {
    let retention_days = retention_days.filter(|&d| d > 0);
    let max_entries = max_entries.filter(|&m| m > 0);

    if retention_days.is_none() && max_entries.is_none() {
        info!("no retention policy configured, cleanup task disabled");
        return;
    }

    info!(
        retention_days = retention_days,
        max_entries = max_entries,
        "starting background cleanup task"
    );

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(CLEANUP_INTERVAL_SECS));

        loop {
            interval.tick().await;

            if let Some(days) = retention_days {
                match cleanup_by_age(&pool, days).await {
                    Ok(0) => {}
                    Ok(count) => info!(count, days, "deleted entries older than retention period"),
                    Err(e) => error!(error = %e, "failed to clean up old entries"),
                }
            }

            if let Some(max) = max_entries {
                match cleanup_by_count(&pool, max).await {
                    Ok(0) => {}
                    Ok(count) => info!(count, max, "deleted entries beyond max count"),
                    Err(e) => error!(error = %e, "failed to clean up excess entries"),
                }
            }
        }
    });
}

async fn cleanup_by_age(pool: &SqlitePool, days: u32) -> Result<u64, sqlx::Error> {
    let cutoff = Utc::now() - chrono::Duration::days(i64::from(days));
    crate::models::Entry::delete_older_than(pool, cutoff).await
}

async fn cleanup_by_count(pool: &SqlitePool, max: u32) -> Result<u64, sqlx::Error> {
    crate::models::Entry::delete_beyond_limit(pool, max).await
}
