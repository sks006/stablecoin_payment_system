use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use crate::config::Settings;
use crate::api::http::handlers;

pub async fn start(settings: &Settings) -> Result<(), crate::domain::error::Error> {
    let app = Router::new()
        .route("/health", get(handlers::health::handle))
        .route("/api/v1/mint", post(handlers::mint::handle))
        .route("/api/v1/transfer", post(handlers::transfer::handle))
        .route("/api/v1/burn", post(handlers::burn::handle))
        .route("/api/v1/webhook", post(handlers::webhook::handle));

    let addr = SocketAddr::from(([0, 0, 0, 0], settings.port));
    tracing::info!("Server running on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .map_err(|e| crate::domain::error::Error::Infrastructure(e.to_string()))?;

    Ok(())
}
