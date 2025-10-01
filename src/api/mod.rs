use axum::{
    Router,
};
use std::sync::Arc;
use crate::AppState;
use crate::config::ApiPaths;

pub mod routes;
pub mod openapi;

pub fn app() -> Router<Arc<AppState>> {
    Router::new().nest(ApiPaths::V1_PREFIX, routes::router())
}
