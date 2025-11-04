use std::{net::IpAddr, sync::Arc};

use axum::{
    Json,
    extract::{State},
    Extension,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use chrono::Utc;
use tracing::{debug, info, instrument, warn};

use crate::{
    config::{ApiPaths, CONFIG, MailConfig},
    services::send_email,
    state::AppState,
    types::{ApiError, ApiMessage, RequestPayload},
    utils::mask_string::mask_email,
};

#[utoipa::path(
    post,
    path = String::from(ApiPaths::V1_PREFIX) + ApiPaths::REQUEST,
    request_body = RequestPayload,
    responses(
        (status = 200, description = "Accepted", body = ApiMessage),
        (status = 400, description = "Bad request", body = ApiError),
        (status = 429, description = "Too many requests", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    ),
    tag = "requests"
)]

#[allow(unused_variables)]
#[instrument(skip(state, payload, headers), fields(ip = %client_ip))]
pub async fn handle_request(
    Extension(client_ip): Extension<IpAddr>,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    payload: Result<Json<RequestPayload>, axum::extract::rejection::JsonRejection>,
) -> axum::response::Response {
    let ip = client_ip.to_string();
    let now = Utc::now().timestamp();

    if CONFIG.use_rate_limit {
        let mut flood = state.flood_control.lock().await;
        let entry = flood.entry(ip.clone()).or_default();
        let before = entry.len();

        entry.retain(|&t| now - t < 3600);
        let after = entry.len();
        if before != after {
            debug!(removed = (before - after), "pruned");
        }
        if entry.len() >= CONFIG.rate_limit_max as usize {
            info!(ip = %ip, "rate limit exceeded");
            return (
                StatusCode::TOO_MANY_REQUESTS,
                Json(ApiError {
                    error: "TooManyRequests".into(),
                    message: "You have reached the maximum number of requests per hour. Please try again later.".into(),
                })
            ).into_response();
        }
        entry.push(now);
    }

    let Json(payload) = match payload {
        Ok(p) => p,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiError {
                    error: "InvalidJson".into(),
                    message: e.to_string(),
                }),
            )
                .into_response();
        }
    };

    if payload.subject.len() > MailConfig::MAX_SUBJECT {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                error: "SubjectTooLong".into(),
                message: format!("subject exceeds {} characters", MailConfig::MAX_SUBJECT),
            }),
        )
            .into_response();
    }
    if payload.message.len() > MailConfig::MAX_MESSAGE {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                error: "MessageTooLong".into(),
                message: format!("message exceeds {} characters", MailConfig::MAX_MESSAGE),
            }),
        )
            .into_response();
    }

    let mut recipients: Vec<String> = Vec::new();

    if CONFIG.allow_email_input {
        if let Some(list) = &payload.recipients {
            recipients.extend(list.iter().cloned());
        }
    } else if payload.recipients.is_some() {
        warn!("recipients field ignored - allow_email_input is disabled");
    }

    let default_emails: &[String] = CONFIG.emails.as_deref().unwrap_or(&[]);

    if CONFIG.duplicate_emails_to_deafult_recipients_everytime {
        recipients.extend(default_emails.iter().cloned());
    } else if recipients.is_empty() {
        recipients.extend(default_emails.iter().cloned());
    }

    recipients.retain(|s| !s.trim().is_empty());
    recipients.sort();
    recipients.dedup();

    if recipients.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                error: "NoRecipient".into(),
                message: "No recipient configured".into(),
            }),
        )
            .into_response();
    }

    let mut failures: Vec<(String, String)> = Vec::new();
    let mut handles = Vec::with_capacity(recipients.len());

    for r in recipients.iter().cloned() {
        debug!(recipient = mask_email(&r), "sending email");
        let subject = payload.subject.clone();
        let message = payload.message.clone();

        handles.push(tokio::task::spawn_blocking(move || {
            (r.clone(), send_email(&r, &subject, &message))
        }));
    }

    for h in handles {
        match h.await {
            Ok((rcpt, res)) => match res {
                Ok(_) => debug!(recipient = mask_email(&rcpt), "email sent"),
                Err(e) => {
                    warn!(recipient = mask_email(&rcpt), error=%e, "send failed");
                    failures.push((rcpt, e.to_string()));
                }
            },
            Err(join_err) => {
                warn!(error=%join_err, "send task join failed");
            }
        }
    }

    if !failures.is_empty() {
        let (failed_count, total) = (failures.len(), recipients.len());
        let detail = failures
            .iter()
            .take(3)
            .map(|(addr, err)| format!("{addr}: {err}"))
            .collect::<Vec<_>>()
            .join("; ");

        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                error: "EmailSendFailed".into(),
                message: format!(
                    "Failed to send {failed_count} of {total} email(s). {}{}",
                    if failed_count > 3 {
                        "Some errors omitted. "
                    } else {
                        ""
                    },
                    detail
                ),
            }),
        )
            .into_response();
    }

    info!(ip = %ip, "accepted");
    (
        StatusCode::OK,
        Json(ApiMessage {
            message: "ok".into(),
        }),
    )
        .into_response()
}
