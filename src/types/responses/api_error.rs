use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema, Clone)]
pub enum ErrorCode {
    InvalidJson,
    SubjectTooLong,
    MessageTooLong,
    NoRecipient,
    TooManyRequests,
    NotFound,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: ErrorCode,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
}

impl ErrorResponse {
    pub fn new(error: ErrorCode, message: String) -> Self {
        Self {
            error,
            message,
            timestamp: Some(chrono::Utc::now().to_rfc3339()),
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct EmailSendResponse {
    pub success: bool,
    pub sent_count: usize,
    pub failed_count: usize,
    pub total_recipients: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub failed_recipients: Vec<FailedRecipient>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    pub timestamp: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct FailedRecipient {
    pub recipient: String,
    pub error: String,
}

impl EmailSendResponse {
    pub fn new(sent_count: usize, failed_count: usize, total: usize) -> Self {
        Self {
            success: failed_count == 0,
            sent_count,
            failed_count,
            total_recipients: total,
            failed_recipients: Vec::new(),
            message: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    pub fn with_failures(sent_count: usize, failed_count: usize, total: usize, failed_recipients: Vec<FailedRecipient>) -> Self {
        Self {
            success: false,
            sent_count,
            failed_count,
            total_recipients: total,
            failed_recipients,
            message: Some(format!("Failed to send {} of {} emails", failed_count, total)),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    pub fn with_masked_failures(sent_count: usize, failed_count: usize, total: usize) -> Self {
        Self {
            success: false,
            sent_count,
            failed_count,
            total_recipients: total,
            failed_recipients: Vec::new(), // Don't expose recipient information
            message: Some("Email sending failed".to_string()),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
}