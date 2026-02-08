use axum::{
    extract::{Request, State},
    middleware::Next,
    response::IntoResponse,
};

use crate::{AppState, errors::Error};

pub async fn auth_guard(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, Error> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .ok_or(Error::Unauthorized)?
        .to_str()
        .map_err(|_| Error::Unauthorized)?;

    if !auth_header.starts_with("Bearer ") {
        return Err(Error::Unauthorized);
    }

    let token = auth_header.trim_start_matches("Bearer ").trim();

    if token != state.config.auth_token {
        return Err(Error::Unauthorized);
    }

    Ok(next.run(req).await)
}
