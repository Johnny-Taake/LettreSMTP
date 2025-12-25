use std::{net::IpAddr, sync::Arc};

use axum::{
    Json,
    extract::{State},
    Extension,
    http::HeaderMap,
};
use chrono::Utc;
use tracing::{debug, info, instrument, warn};

use crate::{
    api::responses,
    config::{ApiPaths, CONFIG, MailConfig},
    services::send_email,
    state::AppState,
    types::{ErrorResponse, ErrorCode, EmailSendResponse, FailedRecipient, RequestPayload},
    utils::mask_string::mask_email,
};

#[utoipa::path(
    post,
    path = String::from(ApiPaths::V1_PREFIX) + ApiPaths::REQUEST,
    request_body = RequestPayload,
    responses(
        (status = 200, description = "Email sent successfully", body = EmailSendResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 429, description = "Too many requests", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "requests",
    description = "Send an email. Note: The 'recipients' field in the request is only used when the 'allow_email_input' configuration is enabled. When disabled, emails are sent only to default configured recipients."
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
            return responses::too_many_requests(ErrorResponse::new(
                ErrorCode::TooManyRequests,
                "You have reached the maximum number of requests per hour. Please try again later.".into(),
            ));
        }
        entry.push(now);
    }

    let Json(payload) = match payload {
        Ok(p) => p,
        Err(e) => {
            return responses::bad_request(ErrorResponse::new(
                ErrorCode::InvalidJson,
                e.to_string(),
            ));
        }
    };

    if payload.subject.len() > MailConfig::MAX_SUBJECT {
        return responses::bad_request(ErrorResponse::new(
            ErrorCode::SubjectTooLong,
            format!("subject exceeds {} characters", MailConfig::MAX_SUBJECT),
        ));
    }
    if payload.message.len() > MailConfig::MAX_MESSAGE {
        return responses::bad_request(ErrorResponse::new(
            ErrorCode::MessageTooLong,
            format!("message exceeds {} characters", MailConfig::MAX_MESSAGE),
        ));
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

    if CONFIG.duplicate_emails_to_deafult_recipients_everytime || recipients.is_empty() {
        recipients.extend(default_emails.iter().cloned());
    }

    recipients.retain(|s| !s.trim().is_empty());
    recipients.sort();
    recipients.dedup();

    if recipients.is_empty() {
        return responses::bad_request(ErrorResponse::new(
            ErrorCode::NoRecipient,
            "No recipient configured".into(),
        ));
    }

    let mut failures: Vec<(String, String)> = Vec::new();
    let mut handles = Vec::with_capacity(recipients.len());

    for r in &recipients {
        debug!(recipient = mask_email(r), "sending email");
        let r = r.clone();
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

    let sent_count = recipients.len() - failures.len();
    let failed_count = failures.len();
    let total_count = recipients.len();

    if !failures.is_empty() {
        // If email input is disabled, don't expose recipient information in the response
        if CONFIG.allow_email_input {
            let failed_recipients: Vec<FailedRecipient> = failures
                .iter()
                .take(10) // Limit the number of detailed failures to avoid huge responses
                .map(|(recipient, error)| FailedRecipient {
                    recipient: recipient.clone(),
                    error: error.clone(),
                })
                .collect();

            let error_response = EmailSendResponse::with_failures(
                sent_count,
                failed_count,
                total_count,
                failed_recipients,
            );

            return responses::internal_server_error(error_response);
        } else {
            let error_response = EmailSendResponse::with_masked_failures(
                sent_count,
                failed_count,
                total_count,
            );

            return responses::internal_server_error(error_response);
        }
    }

    info!(ip = %ip, "accepted");
    let success_response = EmailSendResponse::new(sent_count, failed_count, total_count);
    responses::ok(success_response)
}
