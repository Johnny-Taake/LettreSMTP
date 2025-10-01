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
            crate::types::ApiError,
            crate::types::HealthResponse
        )
    ),
    tags(
        (name = "health"),
        (name = "requests")
    )
)]
pub struct ApiDoc;
