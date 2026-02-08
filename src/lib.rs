use std::sync::Arc;

use axum::{middleware, routing::get};
use sqlx::{SqlitePool, migrate, sqlite::SqlitePoolOptions};
use tower_http::trace::TraceLayer;
use tracing::info;
use utoipa::{
    Modify, OpenApi,
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
};
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_scalar::{Scalar, Servable};

use crate::config::Config;

mod auth;
pub mod config;
mod dto;
mod errors;
mod feed;
mod handlers;
mod metadata;
mod models;

pub const COMMON_TAG: &str = "Common";
pub const FEED_TAG: &str = "Feed";

#[derive(OpenApi)]
#[openapi(
    modifiers(&SecurityAddon),
    tags(
        (name = COMMON_TAG),
        (name = FEED_TAG),
    ),
    security()
)]
pub struct ApiDoc;

pub struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer",
                SecurityScheme::Http(HttpBuilder::new().scheme(HttpAuthScheme::Bearer).build()),
            )
        }
    }
}

pub type AppState = Arc<AppStateInner>;

pub struct AppStateInner {
    pub config: Config,
    pub pool: SqlitePool,
}

pub async fn app(
    config: Config,
    cors_allowed_origins: Vec<String>,
) -> (axum::Router, utoipa::openapi::OpenApi) {
    let cors = tower_http::cors::CorsLayer::new()
        .allow_methods(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any)
        .allow_origin(
            cors_allowed_origins
                .iter()
                .map(|origin| origin.parse().unwrap())
                .collect::<Vec<_>>(),
        );

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await
        .expect("Failed to connect to database");

    info!("migrating database");
    migrate!("./migrations")
        .run(&pool)
        .await
        .expect("failed to migrate database");

    let app_state = AppState::new(AppStateInner { config, pool });

    let authenticated_routes = OpenApiRouter::new()
        .routes(routes!(handlers::add_entry))
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            auth::auth_guard,
        ));

    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .route("/health", get(handlers::health))
        .routes(routes!(handlers::get_feed))
        .routes(routes!(handlers::list_entries))
        .merge(authenticated_routes)
        .with_state(app_state)
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .split_for_parts();

    let router = router.merge(Scalar::with_url("/docs", api.clone()));

    (router, api)
}
