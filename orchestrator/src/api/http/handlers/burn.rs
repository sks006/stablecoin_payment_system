use axum::{Json, response::IntoResponse, http::StatusCode};
use crate::api::http::dto::request::BurnRequest;
use crate::api::http::dto::response::PaymentResponse;
use uuid::Uuid;

pub async fn handle(Json(payload): Json<BurnRequest>) -> impl IntoResponse {
    let response = PaymentResponse {
        id: Uuid::new_v4(),
        idempotency_key: payload.idempotency_key,
        status: "pending".to_string(),
        signature: None,
    };
    (StatusCode::ACCEPTED, Json(response))
}
