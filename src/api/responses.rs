//! Provides standardized HTTP response helpers for API endpoints.
//!
//! This module contains utility functions that wrap response data with appropriate
//! HTTP status codes and JSON serialization for consistent API response formatting.
//!
//! # Available Helpers
//!
//! - [`ok`] — 200 OK response with serialized data  
//! - [`bad_request`] — 400 Bad Request error response  
//! - [`too_many_requests`] — 429 Too Many Requests error response  
//! - [`internal_server_error`] — 500 Internal Server Error response  
//! - [`not_found`] — 404 Not Found error response  
//!
//! # Example
//!
//! ```ignore
//! let response = ok(json!({ "status": "success" }));
//!
//! let error = bad_request(ErrorResponse {
//!     message: "Invalid input".to_string(),
//! });
//! ```

use axum::{
    http::StatusCode,
    response::{Response, IntoResponse},
    Json,
};
use crate::types::{ErrorResponse, EmailSendResponse};

/// Returns a **200 OK** response with JSON-serialized data.
///
/// # Example
/// ```ignore
/// ok(MyData { value: 123 });
/// ```
pub fn ok<T: serde::Serialize>(data: T) -> Response {
    (StatusCode::OK, Json(data)).into_response()
}

/// Returns a **400 Bad Request** error response.
pub fn bad_request(error: ErrorResponse) -> Response {
    (StatusCode::BAD_REQUEST, Json(error)).into_response()
}

/// Returns a **429 Too Many Requests** error response.
pub fn too_many_requests(error: ErrorResponse) -> Response {
    (StatusCode::TOO_MANY_REQUESTS, Json(error)).into_response()
}

/// Returns a **500 Internal Server Error** response.
///
/// Typically used for unexpected backend failures or failed email sending.
pub fn internal_server_error(response: EmailSendResponse) -> Response {
    (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
}

/// Returns a **404 Not Found** error response.
pub fn not_found(error: ErrorResponse) -> Response {
    (StatusCode::NOT_FOUND, Json(error)).into_response()
}
