use axum::Router;
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
use config::{ApiPaths, CONFIG};
use state::AppState;
use std::time::Duration;

#[tokio::main]
async fn main() {
    CONFIG.init_tracing();
    CONFIG.log_effective();

    let state = Arc::new(AppState {
        flood_control: Default::default(),
    });

    let openapi = api::openapi::ApiDoc::openapi();
    let app: Router<_> = api::app()
        .merge(SwaggerUi::new(ApiPaths::SWAGGER_UI).url(ApiPaths::OPENAPI_JSON, openapi))
        .layer(TraceLayer::new_for_http())
        .with_state(state.clone());

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

async fn shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};
        let mut term = signal(SignalKind::terminate()).expect("listen SIGTERM");
        let mut int = signal(SignalKind::interrupt()).expect("listen SIGINT");

        tokio::select! {
            _ = term.recv() => tracing::info!("SIGTERM received"),
            _ = int.recv()  => tracing::info!("SIGINT received"),
        }
    }

    #[cfg(not(unix))]
    {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    }
}
