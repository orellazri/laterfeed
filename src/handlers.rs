use axum::{Json, extract::State, response::IntoResponse};
use axum_valid::Valid;

use crate::{
    AppState, FEED_TAG,
    dto::{AddEntryRequest, EntryResponse, ListEntriesResponse},
    errors::Result,
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
    let entry = models::Entry::create(
        &state.pool,
        &body.url,
        &body.title.unwrap_or("Untitled".to_string()),
        body.summary.as_deref(),
        models::EntrySourceType::Article,
    )
    .await?;

    Ok(Json(EntryResponse::from(entry)))
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
