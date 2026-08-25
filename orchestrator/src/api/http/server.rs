use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::broadcast;
use crate::application::mint_service::MintService;
use crate::api::http::handlers;
use crate::api::http::middleware;

#[derive(Clone)]
pub struct AppState {
    pub mint_service: Arc<MintService>,
}

pub async fn start(
    port: u16,
    mint_service: Arc<MintService>,
    mut shutdown_rx: broadcast::Receiver<()>,
) -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new()
        .route("/health", get(handlers::health::handle))
        .route("/mint", post(handlers::mint::handle))
        .with_state(AppState { mint_service })
        .layer(axum::middleware::from_fn(middleware::auth::require_valid_auth)) // innermost
        .layer(axum::middleware::from_fn(middleware::rate_limiter::rate_limit))
        .layer(axum::middleware::from_fn(middleware::request_id::add_request_id)); // outermost

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("HTTP server listening on http://{}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(async move {
            let _ = shutdown_rx.recv().await;
            tracing::info!("Graceful shutdown initiated: no new requests accepted");
        })
        .await?;

    Ok(())
}
