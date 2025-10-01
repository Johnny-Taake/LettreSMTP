use serde::Deserialize;
use utoipa::ToSchema;

#[allow(dead_code)]
#[derive(Debug, Deserialize, ToSchema)]
pub struct RequestPayload {
    pub subject: String,
    pub message: String,
    #[schema(example = json!(["team@example.com","me@example.com"]))]
    pub recipients: Option<Vec<String>>,
}
