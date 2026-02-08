use std::time::Duration;

use scraper::{Html, Selector};

pub struct PageMetadata {
    pub title: Option<String>,
    pub summary: Option<String>,
}

/// Fetch metadata (title and description) from a URL by downloading and parsing the HTML.
/// Returns `None` values on any failure (network error, parse error, missing elements).
pub async fn fetch_metadata(url: &str) -> PageMetadata {
    match fetch_metadata_inner(url).await {
        Ok(meta) => meta,
        Err(e) => {
            tracing::warn!("Failed to fetch metadata from {}: {}", url, e);
            PageMetadata {
                title: None,
                summary: None,
            }
        }
    }
}

async fn fetch_metadata_inner(url: &str) -> anyhow::Result<PageMetadata> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()?;

    let response = client
        .get(url)
        .header("User-Agent", "Laterfeed/1.0")
        .send()
        .await?
        .error_for_status()?;

    let html = response.text().await?;
    let document = Html::parse_document(&html);

    let title = extract_title(&document);
    let summary = extract_description(&document);

    Ok(PageMetadata { title, summary })
}

fn extract_title(document: &Html) -> Option<String> {
    // Try <meta property="og:title"> first, then fall back to <title>
    let og_title_selector = Selector::parse(r#"meta[property="og:title"]"#).ok()?;
    if let Some(element) = document.select(&og_title_selector).next()
        && let Some(content) = element.value().attr("content")
    {
        let trimmed = content.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }

    let title_selector = Selector::parse("title").ok()?;
    document
        .select(&title_selector)
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .filter(|t| !t.is_empty())
}

fn extract_description(document: &Html) -> Option<String> {
    // Try <meta property="og:description"> first, then <meta name="description">
    let og_desc_selector = Selector::parse(r#"meta[property="og:description"]"#).ok()?;
    if let Some(element) = document.select(&og_desc_selector).next()
        && let Some(content) = element.value().attr("content")
    {
        let trimmed = content.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }

    let desc_selector = Selector::parse(r#"meta[name="description"]"#).ok()?;
    if let Some(element) = document.select(&desc_selector).next()
        && let Some(content) = element.value().attr("content")
    {
        let trimmed = content.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }

    None
}
