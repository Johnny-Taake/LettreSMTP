pub mod logger;
mod responses;
pub use responses::{ApiMessage, HealthResponse, ErrorResponse, ErrorCode, EmailSendResponse, FailedRecipient};
mod requests;
pub use requests::RequestPayload;
