use std::time::Duration;

use scraper::{Html, Selector};
use url::Url;

pub struct PageMetadata {
    pub title: Option<String>,
    pub body: Option<String>,
}

#[derive(serde::Deserialize)]
struct YouTubeOEmbed {
    title: String,
}

/// Fetch metadata (title and body content) from a URL by downloading and parsing the HTML.
/// Uses YouTube oEmbed API for YouTube URLs to reliably extract video titles.
/// Returns `None` values on any failure (network error, parse error, missing elements).
pub async fn fetch_metadata(url: &str) -> PageMetadata {
    if is_youtube_url(url) {
        return fetch_youtube_metadata(url).await;
    }

    match fetch_metadata_inner(url).await {
        Ok(meta) => meta,
        Err(e) => {
            tracing::warn!("Failed to fetch metadata from {}: {}", url, e);
            PageMetadata {
                title: None,
                body: None,
            }
        }
    }
}

/// YouTube pages often return broken titles (e.g. "- YouTube") when scraped directly.
/// Use the oEmbed API for the title and regular HTML scrape for body, in parallel.
async fn fetch_youtube_metadata(url: &str) -> PageMetadata {
    let (oembed_result, page_result) = tokio::join!(
        fetch_youtube_oembed(url),
        fetch_metadata_inner(url),
    );

    let (page_title, body) = match page_result {
        Ok(meta) => (meta.title, meta.body),
        Err(_) => (None, None),
    };

    let title = match oembed_result {
        Ok(oembed) => Some(oembed.title),
        Err(e) => {
            tracing::warn!("Failed to fetch YouTube oEmbed for {}: {}", url, e);
            page_title
        }
    };

    PageMetadata { title, body }
}

async fn fetch_youtube_oembed(url: &str) -> Result<YouTubeOEmbed, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()?;

    let mut oembed_url = Url::parse("https://www.youtube.com/oembed")?;
    oembed_url.query_pairs_mut()
        .append_pair("url", url)
        .append_pair("format", "json");

    let response = client
        .get(oembed_url.as_str())
        .send()
        .await?
        .error_for_status()?;

    let text = response.text().await?;
    Ok(serde_json::from_str(&text)?)
}

fn is_youtube_url(url_str: &str) -> bool {
    let Ok(parsed) = Url::parse(url_str) else {
        return false;
    };

    matches!(
        parsed.host_str(),
        Some(
            "youtube.com"
                | "www.youtube.com"
                | "m.youtube.com"
                | "music.youtube.com"
                | "youtu.be"
        )
    )
}

async fn fetch_metadata_inner(url: &str) -> Result<PageMetadata, Box<dyn std::error::Error + Send + Sync>> {
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
    let body = extract_body(&document);

    Ok(PageMetadata { title, body })
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

/// Extract the page body content as HTML.
/// Tries `<article>` first, then falls back to og:description / meta description.
fn extract_body(document: &Html) -> Option<String> {
    // Try to get the inner HTML of an <article> element
    if let Ok(article_selector) = Selector::parse("article")
        && let Some(article) = document.select(&article_selector).next()
    {
        let html = article.inner_html();
        let trimmed = html.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }

    // Fall back to og:description
    if let Ok(og_desc_selector) = Selector::parse(r#"meta[property="og:description"]"#)
        && let Some(element) = document.select(&og_desc_selector).next()
        && let Some(content) = element.value().attr("content")
    {
        let trimmed = content.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }

    // Fall back to <meta name="description">
    if let Ok(desc_selector) = Selector::parse(r#"meta[name="description"]"#)
        && let Some(element) = document.select(&desc_selector).next()
        && let Some(content) = element.value().attr("content")
    {
        let trimmed = content.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(html: &str) -> Html {
        Html::parse_document(html)
    }

    // --- is_youtube_url tests ---

    #[test]
    fn is_youtube_url_www() {
        assert!(is_youtube_url("https://www.youtube.com/watch?v=dQw4w9WgXcQ"));
    }

    #[test]
    fn is_youtube_url_no_www() {
        assert!(is_youtube_url("https://youtube.com/watch?v=dQw4w9WgXcQ"));
    }

    #[test]
    fn is_youtube_url_mobile() {
        assert!(is_youtube_url("https://m.youtube.com/watch?v=dQw4w9WgXcQ"));
    }

    #[test]
    fn is_youtube_url_music() {
        assert!(is_youtube_url("https://music.youtube.com/watch?v=dQw4w9WgXcQ"));
    }

    #[test]
    fn is_youtube_url_short_link() {
        assert!(is_youtube_url("https://youtu.be/dQw4w9WgXcQ"));
    }

    #[test]
    fn is_youtube_url_shorts() {
        assert!(is_youtube_url("https://www.youtube.com/shorts/dQw4w9WgXcQ"));
    }

    #[test]
    fn is_youtube_url_embed() {
        assert!(is_youtube_url("https://www.youtube.com/embed/dQw4w9WgXcQ"));
    }

    #[test]
    fn is_youtube_url_live() {
        assert!(is_youtube_url("https://www.youtube.com/live/dQw4w9WgXcQ"));
    }

    #[test]
    fn is_youtube_url_http() {
        assert!(is_youtube_url("http://www.youtube.com/watch?v=dQw4w9WgXcQ"));
    }

    #[test]
    fn is_youtube_url_with_extra_params() {
        assert!(is_youtube_url(
            "https://www.youtube.com/watch?v=dQw4w9WgXcQ&t=120&list=PLrAXtmErZgOeiKm4sgNOknGvNjby9efdf"
        ));
    }

    #[test]
    fn is_youtube_url_rejects_non_youtube() {
        assert!(!is_youtube_url("https://example.com/watch?v=dQw4w9WgXcQ"));
    }

    #[test]
    fn is_youtube_url_rejects_similar_domains() {
        assert!(!is_youtube_url("https://notyoutube.com/watch?v=abc"));
        assert!(!is_youtube_url("https://youtube.com.evil.com/watch?v=abc"));
    }

    #[test]
    fn is_youtube_url_rejects_invalid_url() {
        assert!(!is_youtube_url("not a url"));
    }

    #[test]
    fn is_youtube_url_channel_page() {
        assert!(is_youtube_url("https://www.youtube.com/@somechannel"));
    }

    // --- extract_title tests ---

    #[test]
    fn extract_title_from_og_title() {
        let doc = parse(r#"<html><head><meta property="og:title" content="OG Title"></head></html>"#);
        assert_eq!(extract_title(&doc), Some("OG Title".to_string()));
    }

    #[test]
    fn extract_title_from_title_tag() {
        let doc = parse("<html><head><title>Page Title</title></head></html>");
        assert_eq!(extract_title(&doc), Some("Page Title".to_string()));
    }

    #[test]
    fn extract_title_og_takes_precedence_over_title_tag() {
        let doc = parse(
            r#"<html><head>
            <meta property="og:title" content="OG Title">
            <title>Page Title</title>
            </head></html>"#,
        );
        assert_eq!(extract_title(&doc), Some("OG Title".to_string()));
    }

    #[test]
    fn extract_title_falls_back_to_title_when_og_empty() {
        let doc = parse(
            r#"<html><head>
            <meta property="og:title" content="">
            <title>Fallback Title</title>
            </head></html>"#,
        );
        assert_eq!(extract_title(&doc), Some("Fallback Title".to_string()));
    }

    #[test]
    fn extract_title_falls_back_to_title_when_og_whitespace() {
        let doc = parse(
            r#"<html><head>
            <meta property="og:title" content="   ">
            <title>Fallback Title</title>
            </head></html>"#,
        );
        assert_eq!(extract_title(&doc), Some("Fallback Title".to_string()));
    }

    #[test]
    fn extract_title_none_when_no_title() {
        let doc = parse("<html><head></head><body>Hello</body></html>");
        assert_eq!(extract_title(&doc), None);
    }

    #[test]
    fn extract_title_none_when_empty_title_tag() {
        let doc = parse("<html><head><title>   </title></head></html>");
        assert_eq!(extract_title(&doc), None);
    }

    #[test]
    fn extract_title_trims_whitespace() {
        let doc = parse(r#"<html><head><meta property="og:title" content="  Trimmed  "></head></html>"#);
        assert_eq!(extract_title(&doc), Some("Trimmed".to_string()));
    }

    // --- extract_body tests ---

    #[test]
    fn extract_body_from_article_element() {
        let doc = parse(
            r#"<html><body><article><p>Article content</p></article></body></html>"#,
        );
        assert_eq!(
            extract_body(&doc),
            Some("<p>Article content</p>".to_string())
        );
    }

    #[test]
    fn extract_body_article_takes_precedence_over_description() {
        let doc = parse(
            r#"<html><head>
            <meta property="og:description" content="OG Desc">
            </head><body><article><p>Article content</p></article></body></html>"#,
        );
        assert_eq!(
            extract_body(&doc),
            Some("<p>Article content</p>".to_string())
        );
    }

    #[test]
    fn extract_body_falls_back_to_og_description() {
        let doc = parse(
            r#"<html><head><meta property="og:description" content="OG Desc"></head><body><p>Some text</p></body></html>"#,
        );
        assert_eq!(extract_body(&doc), Some("OG Desc".to_string()));
    }

    #[test]
    fn extract_body_falls_back_to_meta_description() {
        let doc = parse(
            r#"<html><head><meta name="description" content="Meta Desc"></head><body><p>Some text</p></body></html>"#,
        );
        assert_eq!(extract_body(&doc), Some("Meta Desc".to_string()));
    }

    #[test]
    fn extract_body_og_description_takes_precedence_over_meta_description() {
        let doc = parse(
            r#"<html><head>
            <meta property="og:description" content="OG Desc">
            <meta name="description" content="Meta Desc">
            </head></html>"#,
        );
        assert_eq!(extract_body(&doc), Some("OG Desc".to_string()));
    }

    #[test]
    fn extract_body_falls_back_when_og_empty() {
        let doc = parse(
            r#"<html><head>
            <meta property="og:description" content="">
            <meta name="description" content="Fallback Desc">
            </head></html>"#,
        );
        assert_eq!(extract_body(&doc), Some("Fallback Desc".to_string()));
    }

    #[test]
    fn extract_body_falls_back_when_og_whitespace() {
        let doc = parse(
            r#"<html><head>
            <meta property="og:description" content="   ">
            <meta name="description" content="Fallback Desc">
            </head></html>"#,
        );
        assert_eq!(extract_body(&doc), Some("Fallback Desc".to_string()));
    }

    #[test]
    fn extract_body_none_when_missing() {
        let doc = parse("<html><head></head><body>Hello</body></html>");
        assert_eq!(extract_body(&doc), None);
    }

    #[test]
    fn extract_body_none_when_both_descriptions_empty() {
        let doc = parse(
            r#"<html><head>
            <meta property="og:description" content="">
            <meta name="description" content="">
            </head></html>"#,
        );
        assert_eq!(extract_body(&doc), None);
    }

    #[test]
    fn extract_body_trims_whitespace() {
        let doc = parse(
            r#"<html><head><meta property="og:description" content="  Trimmed  "></head></html>"#,
        );
        assert_eq!(extract_body(&doc), Some("Trimmed".to_string()));
    }

    #[test]
    fn extract_body_empty_article_falls_back_to_description() {
        let doc = parse(
            r#"<html><head>
            <meta property="og:description" content="OG Desc">
            </head><body><article>   </article></body></html>"#,
        );
        assert_eq!(extract_body(&doc), Some("OG Desc".to_string()));
    }
}
