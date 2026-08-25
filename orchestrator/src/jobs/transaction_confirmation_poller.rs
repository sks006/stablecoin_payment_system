// orchestrator/src/jobs/transaction_confirmation_poller.rs

use crate::{
    domain::payment::PaymentStatus,
    infrastructure::db::repositories::payment_repo::PaymentRepository,
    application::webhook_dispatcher::WebhookDispatcher,
};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    signature::Signature,
};
use std::str::FromStr;
use std::sync::Arc;
use tokio::{
    sync::broadcast,
    time::Duration,
};
use tracing::{debug, error, info, warn};

/// Background poller that continuously monitors pending transactions
/// and reconciles their final status based on blockchain confirmation.
pub struct TransactionConfirmationPoller {
    payment_repo: Arc<PaymentRepository>,
    rpc_client: Arc<RpcClient>,
    webhook_dispatcher: Arc<WebhookDispatcher>,
    shutdown_rx: broadcast::Receiver<()>,
    poll_interval: Duration,
}

impl TransactionConfirmationPoller {
    pub fn new(
        payment_repo: Arc<PaymentRepository>,
        rpc_client: Arc<RpcClient>,
        webhook_dispatcher: Arc<WebhookDispatcher>,
        shutdown_rx: broadcast::Receiver<()>,
    ) -> Self {
        Self {
            payment_repo,
            rpc_client,
            webhook_dispatcher,
            shutdown_rx,
            poll_interval: Duration::from_millis(500),
        }
    }

    pub async fn run(mut self) {
        loop {
            tokio::select! {
                // [MARKER: Define the graceful shutdown receiver branch]
                _ = self.shutdown_rx.recv() => {
                    break;
                }
                // [MARKER: Define the 500ms sleep and DB sweep branch]
                _ = tokio::time::sleep(self.poll_interval) => {
                    self.sweep_pending_transactions().await;
                }
            }
        }
    }

    /// Fetch all pending transactions and attempt to confirm them.
    async fn sweep_pending_transactions(&self) {
        // 1. Retrieve all records with status SUBMITTED
        let pending_txs = match self.payment_repo.get_submitted_payments().await {
            Ok(txs) => txs,
            Err(e) => {
                error!("Failed to list pending transactions: {:?}", e);
                return;
            }
        };

        if pending_txs.is_empty() {
            debug!("No pending transactions to sweep");
            return;
        }

        info!("Sweeping {} pending transactions", pending_txs.len());

        // 2. For each pending transaction, query the RPC and update status
        for tx in pending_txs {
            let tx_id_str = match &tx.signature {
                Some(sig) => sig,
                None => continue,
            };

            let signature = match Signature::from_str(tx_id_str) {
                Ok(sig) => sig,
                Err(e) => {
                    error!("Invalid signature string {} in database: {:?}", tx_id_str, e);
                    continue;
                }
            };

            match self.rpc_client.get_signature_status_with_commitment(&signature, CommitmentConfig::confirmed()).await {
                Ok(Some(status)) => {
                    if status.is_ok() {
                        if let Err(e) = self.payment_repo.update_status(tx.id, PaymentStatus::Confirmed).await {
                            error!("Failed to mark payment {} as confirmed: {:?}", tx.id, e);
                        } else {
                            debug!("Transaction {} confirmed", tx_id_str);
                            let mut confirmed_payment = tx.clone();
                            confirmed_payment.status = PaymentStatus::Confirmed;
                            confirmed_payment.signature = Some(tx_id_str.clone());
                            if let Err(e) = self.webhook_dispatcher.dispatch(&confirmed_payment, "payment.confirmed").await {
                                error!("Failed to dispatch webhook for payment {}: {:?}", tx.id, e);
                            }
                        }
                    } else {
                        // Transaction failed on-chain
                        warn!("Transaction {} failed: {:?}", tx_id_str, status);
                        if let Err(e) = self.payment_repo.update_status(tx.id, PaymentStatus::Failed).await {
                            error!("Failed to mark payment {} as failed: {:?}", tx.id, e);
                        }
                    }
                }
                Ok(None) => {
                    // Not yet confirmed; do nothing
                    debug!("Transaction {} not yet confirmed", tx_id_str);
                }
                Err(e) => {
                    error!("RPC error while checking {}: {:?}", tx_id_str, e);
                }
            }
        }
    }
}