use axum::{response::IntoResponse, http::StatusCode};

pub async fn handle() -> impl IntoResponse {
    StatusCode::OK
}
