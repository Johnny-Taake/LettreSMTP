use axum::Json;
use chrono::Utc;

use crate::config::ApiPaths;
use crate::types::HealthResponse;

#[utoipa::path(
    get,
    path = String::from(ApiPaths::V1_PREFIX) + ApiPaths::HEALTH,
    responses(
        (status = 200, description = "Service is healthy", body = HealthResponse),
        (status = 500, description = "Internal server error", body = crate::types::ApiError)
    ),
    tag = "health"
)]
pub async fn handle_health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        timestamp: Utc::now().to_rfc3339(),
    })
}
