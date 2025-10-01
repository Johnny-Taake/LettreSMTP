use axum::{routing::{get, post}, Router, extract::OriginalUri, http::Method, response::IntoResponse, Json, http::StatusCode};
use std::sync::Arc;

use crate::state::AppState;
use crate::config::ApiPaths;
use crate::types::ApiError;

pub mod health;
pub mod request;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route(ApiPaths::HEALTH, get(health::handle_health))
        .route(ApiPaths::REQUEST, post(request::handle_request))
        .fallback(api_not_found)
}

async fn api_not_found(OriginalUri(uri): OriginalUri, method: Method) -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        Json(ApiError {
            error: "NotFound".into(),
            message: format!("No route for {} {}", method, uri.path()),
        }),
    )
}
