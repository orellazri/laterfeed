use axum::{
    Json,
    extract::State,
    http::{StatusCode, header},
    response::IntoResponse,
};
use axum_valid::Valid;

use crate::{
    AppState, FEED_TAG,
    dto::{AddEntryRequest, EntryResponse, ListEntriesResponse},
    errors::Result,
    feed,
    metadata,
    models,
};

pub async fn health() -> &'static str {
    "ok"
}

#[utoipa::path(
    post,
    path = "/entries",
    summary = "Add an entry",
    operation_id = "addEntry",
    tag = FEED_TAG,
    responses(
        (status = 201, description = "Entry", body = EntryResponse),
    ),
    security(),
)]
pub async fn add_entry(
    State(state): State<AppState>,
    Valid(Json(body)): Valid<Json<AddEntryRequest>>,
) -> Result<impl IntoResponse> {
    let mut title = body.title;
    let mut summary = body.summary;

    // If title or summary are missing, try to fetch them from the page
    if title.is_none() || summary.is_none() {
        let meta = metadata::fetch_metadata(&body.url).await;
        if title.is_none() {
            title = meta.title;
        }
        if summary.is_none() {
            summary = meta.summary;
        }
    }

    // Fall back to using the URL as the title if still missing
    let title = title.unwrap_or_else(|| body.url.clone());

    // Use caller-provided source_type, or auto-detect from URL
    let source_type = body
        .source_type
        .map(|st| st.into())
        .unwrap_or_else(|| metadata::detect_source_type(&body.url));

    let entry = models::Entry::create(
        &state.pool,
        &body.url,
        &title,
        summary.as_deref(),
        source_type,
    )
    .await?;

    Ok((StatusCode::CREATED, Json(EntryResponse::from(entry))))
}

#[utoipa::path(
    get,
    path = "/entries",
    summary = "List entries",
    operation_id = "listEntries",
    tag = FEED_TAG,
    responses(
        (status = 200, description = "List of entries", body = ListEntriesResponse),
    ),
    security(),
)]
pub async fn list_entries(State(state): State<AppState>) -> Result<impl IntoResponse> {
    let entries = models::Entry::fetch_all(&state.pool).await?;

    Ok(Json(ListEntriesResponse {
        entries: entries.into_iter().map(|e| e.into()).collect(),
    }))
}

pub async fn get_feed(State(state): State<AppState>) -> Result<impl IntoResponse> {
    let entries = models::Entry::fetch_latest(&state.pool, feed::entry_limit()).await?;
    let xml = feed::build_atom_feed(&entries, &state.config.base_url);

    Ok((
        [(header::CONTENT_TYPE, "application/atom+xml; charset=utf-8")],
        xml,
    ))
}
