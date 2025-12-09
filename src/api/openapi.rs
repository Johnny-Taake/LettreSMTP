use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::api::routes::health::handle_health,
        crate::api::routes::request::handle_request,
    ),
    components(
        schemas(
            crate::types::RequestPayload,
            crate::types::ApiMessage,
            crate::types::HealthResponse,
            crate::types::ErrorResponse,
            crate::types::ErrorCode,
            crate::types::EmailSendResponse,
            crate::types::FailedRecipient
        )
    ),
    tags(
        (name = "health"),
        (name = "requests")
    )
)]
pub struct ApiDoc;
