use serde::Deserialize;
use utoipa::ToSchema;

#[allow(dead_code)]
#[derive(Debug, Deserialize, ToSchema)]
pub struct RequestPayload {
    pub subject: String,
    pub message: String,
    /// Note: This field is ignored when `allow_email_input` configuration is disabled,
    /// and emails will be sent only to default configured recipients.
    #[schema(example = json!(["team@example.com","me@example.com"]))]
    pub recipients: Option<Vec<String>>,
}
