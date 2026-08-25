use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};
use crate::api::http::server::AppState;
use crate::api::http::dto::request::MintRequest;
use crate::api::http::dto::response::MintResponse;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use shared_memory::OrchestratorError;

fn map_error(e: OrchestratorError) -> StatusCode {
    match e {
        OrchestratorError::InvalidPDA
        | OrchestratorError::InvalidAccountOwner
        | OrchestratorError::InvalidAccountSize
        | OrchestratorError::InvalidDiscriminator
        | OrchestratorError::UnalignedMemoryAccess
        | OrchestratorError::InvalidInstructionData
        | OrchestratorError::InvalidPubkey => StatusCode::BAD_REQUEST,
        OrchestratorError::IdempotencyConflict => StatusCode::CONFLICT,
        OrchestratorError::KmsUnavailable => StatusCode::SERVICE_UNAVAILABLE,
        OrchestratorError::KmsTimeout => StatusCode::GATEWAY_TIMEOUT,
        OrchestratorError::SolanaRpcError => StatusCode::BAD_GATEWAY,
        OrchestratorError::BlockhashExpired => StatusCode::CONFLICT,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn handle(
    State(state): State<AppState>,
    _headers: HeaderMap,
    Json(req): Json<MintRequest>,
) -> Result<Json<MintResponse>, (StatusCode, String)> {
    // Parse pubkeys manually, map parse error to 400
    let funder_wallet = Pubkey::from_str(&req.funder_wallet)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid funder_wallet pubkey".to_string()))?;
    let funder_token = Pubkey::from_str(&req.funder_token)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid funder_token pubkey".to_string()))?;
    let vault = Pubkey::from_str(&req.vault)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid vault pubkey".to_string()))?;
    let vault_token = Pubkey::from_str(&req.vault_token)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid vault_token pubkey".to_string()))?;
    let collateral_mint = Pubkey::from_str(&req.collateral_mint)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid collateral_mint pubkey".to_string()))?;

    match state.mint_service.mint_stablecoin(
        req.idempotency_key.clone(),
        funder_wallet,
        funder_token,
        vault,
        vault_token,
        collateral_mint,
        req.debt_amount,
        req.collateral_deposit,
    ).await {
        Ok(signature) => Ok(Json(MintResponse {
            signature,
            status: "submitted".to_string(),
        })),
        Err(e) => {
            tracing::error!("Mint request failed: {:?}", e);
            let orch_err = OrchestratorError::from(e);
            let code = map_error(orch_err);
            Err((code, format!("{:?}", code)))
        }
    }
}
