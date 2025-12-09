use axum::{routing::{get, post}, Router, extract::OriginalUri, http::Method, response::IntoResponse};
use std::sync::Arc;

use crate::{api::responses, state::AppState, config::ApiPaths, types::{ErrorResponse, ErrorCode}};

pub mod health;
pub mod request;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route(ApiPaths::HEALTH, get(health::handle_health))
        .route(ApiPaths::REQUEST, post(request::handle_request))
        .fallback(api_not_found)
}

async fn api_not_found(OriginalUri(uri): OriginalUri, method: Method) -> impl IntoResponse {
    responses::not_found(ErrorResponse::new(
        ErrorCode::NotFound,
        format!("No route for {} {}", method, uri.path()),
    ))
}
