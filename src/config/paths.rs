pub struct ApiPaths;

impl ApiPaths {
    pub const V1_PREFIX: &'static str = "/api/v1";

    pub const REQUEST: &'static str =  "/request";
    pub const HEALTH: &'static str = "/health";

    pub const SWAGGER_UI: &'static str = "/docs";
    pub const OPENAPI_JSON: &'static str = "/openapi.json";
}
