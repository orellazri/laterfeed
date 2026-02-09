use atom_syndication::{Content, Entry as AtomEntry, Feed as AtomFeed, FixedDateTime, Link, Text};
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

    if let Some(ref body) = entry.body {
        atom_entry.content = Some(Content {
            value: Some(body.clone()),
            content_type: Some("html".to_string()),
            ..Default::default()
        });
    }

    atom_entry
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::EntrySourceType;
    use chrono::TimeZone;

    fn make_entry(id: i64, url: &str, title: &str, body: Option<&str>) -> Entry {
        Entry {
            id,
            url: url.to_string(),
            title: title.to_string(),
            body: body.map(|s| s.to_string()),
            source_type: EntrySourceType::Article,
            created_at: Utc.with_ymd_and_hms(2026, 1, 15, 12, 0, 0).unwrap(),
        }
    }

    #[test]
    fn build_atom_feed_empty_entries() {
        let xml = build_atom_feed(&[], "https://example.com");

        assert!(xml.contains("<title>Laterfeed</title>"));
        assert!(xml.contains("<id>https://example.com/feed</id>"));
        assert!(xml.contains(r#"rel="self""#));
        assert!(xml.contains(r#"rel="alternate""#));
        assert!(!xml.contains("<entry>"));
    }

    #[test]
    fn build_atom_feed_multiple_entries_with_and_without_body() {
        let entries = vec![
            make_entry(1, "https://example.com/a", "First", Some("<p>Body A</p>")),
            make_entry(2, "https://example.com/b", "Second", None),
            make_entry(3, "https://example.com/c", "Third", Some("<p>Body C</p>")),
        ];

        let xml = build_atom_feed(&entries, "https://example.com");

        // All entries present
        assert!(xml.contains("<title>First</title>"));
        assert!(xml.contains("<title>Second</title>"));
        assert!(xml.contains("<title>Third</title>"));

        // Body content included where provided
        assert!(xml.contains("Body A"));
        assert!(xml.contains("Body C"));

        // Content type should be html
        assert!(xml.contains(r#"type="html""#));

        // Feed updated time should come from the first entry
        assert!(xml.contains("2026-01-15"));
    }

    #[test]
    fn entry_to_atom_maps_all_fields() {
        let entry = make_entry(
            10,
            "https://example.com/article",
            "Test Article",
            Some("<p>Test body</p>"),
        );

        let atom = entry_to_atom(&entry);

        assert_eq!(atom.title.value, "Test Article");
        assert_eq!(atom.id, "https://example.com/article");
        assert_eq!(atom.links.len(), 1);
        assert_eq!(atom.links[0].href, "https://example.com/article");
        assert_eq!(atom.links[0].rel, "alternate");
        let content = atom.content.unwrap();
        assert_eq!(content.value.unwrap(), "<p>Test body</p>");
        assert_eq!(content.content_type.unwrap(), "html");
    }
}
