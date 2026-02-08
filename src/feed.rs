use atom_syndication::{Entry as AtomEntry, Feed as AtomFeed, FixedDateTime, Link, Text};
use chrono::Utc;

use crate::models::Entry;

const FEED_ENTRY_LIMIT: i64 = 50;

/// Maximum number of entries to include in the feed.
pub fn entry_limit() -> i64 {
    FEED_ENTRY_LIMIT
}

/// Build an Atom feed XML string from a list of entries.
pub fn build_atom_feed(entries: &[Entry], base_url: &str) -> String {
    let updated = entries
        .first()
        .map(|e| e.created_at)
        .unwrap_or_else(Utc::now);

    let feed_link = Link {
        href: format!("{}/feed", base_url),
        rel: "self".to_string(),
        mime_type: Some("application/atom+xml".to_string()),
        ..Default::default()
    };

    let site_link = Link {
        href: base_url.to_string(),
        rel: "alternate".to_string(),
        ..Default::default()
    };

    let atom_entries: Vec<AtomEntry> = entries.iter().map(entry_to_atom).collect();

    let feed = AtomFeed {
        title: Text::plain("Laterfeed"),
        id: format!("{}/feed", base_url),
        updated: FixedDateTime::from(updated),
        links: vec![feed_link, site_link],
        entries: atom_entries,
        ..Default::default()
    };

    feed.to_string()
}

fn entry_to_atom(entry: &Entry) -> AtomEntry {
    let link = Link {
        href: entry.url.clone(),
        rel: "alternate".to_string(),
        ..Default::default()
    };

    let mut atom_entry = AtomEntry {
        title: Text::plain(&entry.title),
        id: entry.url.clone(),
        updated: FixedDateTime::from(entry.created_at),
        links: vec![link],
        ..Default::default()
    };

    if let Some(ref summary) = entry.summary {
        atom_entry.summary = Some(Text::plain(summary));
    }

    atom_entry
}
