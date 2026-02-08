use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use http_body_util::BodyExt;
use laterfeed::config::Config;
use serde_json::{Value, json};
use tower::ServiceExt;

async fn setup_app() -> axum::Router {
    let config = Config {
        port: 0,
        database_url: "sqlite::memory:".to_string(),
        base_url: "http://localhost:3000".to_string(),
        auth_token: "test-token".to_string(),
    };

    let (router, _, _) = laterfeed::app(config).await;
    router
}

// --- Auth ---

#[tokio::test]
async fn add_entry_without_auth_returns_unauthorized() {
    let app = setup_app().await;

    let body = json!({
        "url": "https://example.com",
        "source_type": "article"
    });

    let response = app
        .oneshot(
            Request::post("/entries")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn add_entry_with_wrong_token_returns_unauthorized() {
    let app = setup_app().await;

    let body = json!({
        "url": "https://example.com",
        "source_type": "article"
    });

    let response = app
        .oneshot(
            Request::post("/entries")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, "Bearer wrong-token")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

// --- Add entry ---

#[tokio::test]
async fn add_entry_with_valid_auth_creates_entry() {
    let app = setup_app().await;

    let body = json!({
        "url": "https://example.com/test-article",
        "title": "Test Article",
        "summary": "A test summary",
        "source_type": "article"
    });

    let response = app
        .oneshot(
            Request::post("/entries")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, "Bearer test-token")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["url"], "https://example.com/test-article");
    assert_eq!(json["title"], "Test Article");
    assert_eq!(json["summary"], "A test summary");
    assert_eq!(json["source_type"], "article");
    assert!(json["id"].is_number());
    assert!(json["created_at"].is_string());
}

#[tokio::test]
async fn add_entry_without_title_uses_url_as_fallback() {
    let app = setup_app().await;

    // Use a URL that won't resolve so metadata fetch fails,
    // causing the title to fall back to the URL itself
    let body = json!({
        "url": "https://invalid.nonexistent.example/page",
        "source_type": "article"
    });

    let response = app
        .oneshot(
            Request::post("/entries")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, "Bearer test-token")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["title"], "https://invalid.nonexistent.example/page");
}

// --- List entries ---

#[tokio::test]
async fn list_entries_returns_created_entries() {
    let config = Config {
        port: 0,
        database_url: "sqlite::memory:".to_string(),
        base_url: "http://localhost:3000".to_string(),
        auth_token: "test-token".to_string(),
    };

    let (router, _, _) = laterfeed::app(config).await;

    // Add an entry
    let add_body = json!({
        "url": "https://example.com/listed",
        "title": "Listed Entry",
        "summary": "Should appear in list",
        "source_type": "article"
    });

    let response = router
        .clone()
        .oneshot(
            Request::post("/entries")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, "Bearer test-token")
                .body(Body::from(serde_json::to_string(&add_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    // List should return the entry without auth
    let response = router
        .oneshot(Request::get("/entries").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();

    let entries = json["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0]["url"], "https://example.com/listed");
    assert_eq!(entries[0]["title"], "Listed Entry");
}

// --- Feed ---

#[tokio::test]
async fn get_feed_returns_valid_atom_xml() {
    let app = setup_app().await;

    let response = app
        .oneshot(Request::get("/feed").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let content_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .unwrap()
        .to_str()
        .unwrap();
    assert!(content_type.contains("application/atom+xml"));

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let xml = String::from_utf8(body.to_vec()).unwrap();

    assert!(xml.contains("<title>Laterfeed</title>"));
    assert!(xml.contains("http://localhost:3000/feed"));
}

#[tokio::test]
async fn get_feed_includes_created_entries() {
    let config = Config {
        port: 0,
        database_url: "sqlite::memory:".to_string(),
        base_url: "http://localhost:3000".to_string(),
        auth_token: "test-token".to_string(),
    };

    let (router, _, _) = laterfeed::app(config).await;

    // Add an entry
    let add_body = json!({
        "url": "https://example.com/feed-item",
        "title": "Feed Item",
        "summary": "In the feed",
        "source_type": "article"
    });

    router
        .clone()
        .oneshot(
            Request::post("/entries")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, "Bearer test-token")
                .body(Body::from(serde_json::to_string(&add_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Feed should contain the entry
    let response = router
        .oneshot(Request::get("/feed").body(Body::empty()).unwrap())
        .await
        .unwrap();

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let xml = String::from_utf8(body.to_vec()).unwrap();

    assert!(xml.contains("<title>Feed Item</title>"));
    assert!(xml.contains("https://example.com/feed-item"));
    assert!(xml.contains("In the feed"));
}
