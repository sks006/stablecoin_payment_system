use axum::{Json, response::IntoResponse, http::StatusCode};
use crate::api::http::dto::request::TransferRequest;
use crate::api::http::dto::response::PaymentResponse;
use uuid::Uuid;

pub async fn handle(Json(payload): Json<TransferRequest>) -> impl IntoResponse {
    let response = PaymentResponse {
        id: Uuid::new_v4(),
        idempotency_key: payload.idempotency_key,
        status: "submitted".to_string(),
        signature: Some(payload.signature),
    };
    (StatusCode::ACCEPTED, Json(response))
}
