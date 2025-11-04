use http::{
    HeaderValue, Method,
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
};
use std::time::Duration;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::{debug, error};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use std::{net::SocketAddr, sync::Arc};

mod api;
mod config;
mod errors;
mod services;
mod state;
mod types;
mod utils;
mod middlewares;
mod shutdown;

use config::{ApiPaths, CONFIG};
use state::AppState;
use shutdown::shutdown_signal;

#[tokio::main]
async fn main() {
    CONFIG.init_tracing();
    CONFIG.log_effective();

    let state = Arc::new(AppState {
        flood_control: Default::default(),
    });

    let app = api::app()
        .merge(
            SwaggerUi::new(ApiPaths::SWAGGER_UI)
                .url(ApiPaths::OPENAPI_JSON, api::openapi::ApiDoc::openapi()),
        )
        .layer(TraceLayer::new_for_http())
        .route_layer(axum::middleware::from_fn(middlewares::real_ip_layer))
        .with_state(state.clone());

    let allowed_origins: Vec<HeaderValue> = CONFIG
        .cors_origins
        .as_ref()
        .map(|origins| {
            origins
                .iter()
                .filter_map(|origin| origin.parse().ok())
                .collect()
        })
        .unwrap_or_default();

    let cors = CorsLayer::new()
        .allow_origin(allowed_origins)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([CONTENT_TYPE, AUTHORIZATION, ACCEPT])
        .expose_headers([AUTHORIZATION])
        .max_age(Duration::from_secs(60 * 30));

    let app = app.layer(cors);

    let addr = SocketAddr::from(([0, 0, 0, 0], CONFIG.port));
    debug!(%addr, "listening");

    let handle = axum_server::Handle::new();
    tokio::spawn({
        let handle = handle.clone();
        async move {
            shutdown_signal().await;
            tracing::info!("Shutdown signal received, shutting down gracefully...");
            handle.graceful_shutdown(Some(Duration::from_secs(10)));
        }
    });

    if let Err(e) = axum_server::bind(addr)
        .handle(handle)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
    {
        error!("server error: {e}");
    }
}
