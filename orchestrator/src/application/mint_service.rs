use crate::{
    domain::{
        error::Error as DomainError,
        payment::{Payment, PaymentStatus},
    },
    infrastructure::{
        db::repositories::idempotency_repo::IdempotencyRepository,
        db::repositories::payment_repo::PaymentRepository,
        kms::aws_client::KmsClient,
        solana::tpu_client::TpuClient,
    },
    application::transfer_service::TransactionBuilder,
};
use solana_sdk::{
    pubkey::Pubkey,
    signature::Signature,
    transaction::Transaction,
};
use std::sync::Arc;

pub struct MintService {
    idempotency_repo: Arc<IdempotencyRepository>,
    payment_repo: Arc<PaymentRepository>,
    kms_client: Arc<KmsClient>,
    tpu_client: Arc<TpuClient>,
    builder: TransactionBuilder,
}

impl MintService {
    pub fn new(
        idempotency_repo: Arc<IdempotencyRepository>,
        payment_repo: Arc<PaymentRepository>,
        kms_client: Arc<KmsClient>,
        tpu_client: Arc<TpuClient>,
        builder: TransactionBuilder,
    ) -> Self {
        Self {
            idempotency_repo,
            payment_repo,
            kms_client,
            tpu_client,
            builder,
        }
    }

    pub async fn mint_stablecoin(
        &self,
        idempotency_key: String,
        funder_wallet: Pubkey,
        funder_token: Pubkey,
        vault: Pubkey,
        vault_token: Pubkey,
        collateral_mint: Pubkey,
        debt_amount: u64,
        collateral_deposit: u64,
    ) -> Result<String, DomainError> {

        // === 1. THE ATOMIC LOCK ===
        // Attempt to acquire the idempotency lock. If it already exists,
        // the repository returns LockAlreadyExists and we abort early.
        self.idempotency_repo
            .acquire_lock(&idempotency_key)
            .await?;

        // Create and save a new payment record in the database
        let payment = Payment {
            id: uuid::Uuid::new_v4(),
            idempotency_key: idempotency_key.clone(),
            amount: debt_amount,
            sender: funder_wallet.to_string(),
            recipient: vault.to_string(),
            status: PaymentStatus::Pending,
            signature: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        self.payment_repo.save(&payment).await?;

        // === 2. THE ABI TRANSLATION ===
        // Build the raw 17‑byte instruction with correct account ordering.
        let ix = self.builder
            .build_mint_jit_ix(
                funder_wallet,
                funder_token,
                vault,
                vault_token,
                collateral_mint,
                debt_amount,
                collateral_deposit,
            )
            .map_err(|e| DomainError::Solana(format!("{:?}", e)))?;

        // === 3. BLOCKHASH & TRANSACTION COMPILATION ===
        // Fetch a fresh blockhash from the RPC (via TPU client)
        let recent_blockhash = self.tpu_client.get_latest_blockhash().await?;

        // The payer is the funder wallet (admin key managed by KMS)
        let payer_pubkey = self.kms_client.get_pubkey();

        let mut tx = Transaction::new_with_payer(
            std::slice::from_ref(&ix),
            Some(&payer_pubkey),
        );
        tx.message.recent_blockhash = recent_blockhash;

        // === 4. THE HARDWARE SIGNATURE ===
        // Use KMS to sign the transaction. This returns a single signature
        // for the payer pubkey (which is also the funder wallet).
        let signature: Signature = self
            .kms_client
            .sign_transaction(&tx.message)
            .await?;

        // Attach the signature to the transaction.
        tx.signatures = vec![signature];

        // === 5. THE LEDGER COMMIT ===
        // Compute the transaction ID (signature string) and update the
        // idempotency record before broadcasting.
        let tx_id = signature.to_string();
        self.idempotency_repo
            .update_tx_id(&idempotency_key, &tx_id)
            .await?;

        // Update payment record to Submitted with signature
        let mut submitted_payment = payment.clone();
        submitted_payment.status = PaymentStatus::Submitted;
        submitted_payment.signature = Some(tx_id.clone());
        submitted_payment.updated_at = chrono::Utc::now();
        self.payment_repo.save(&submitted_payment).await?;

        // === 6. NETWORK EGRESS ===
        // Send the signed transaction to the TPU for inclusion in a block.
        self.tpu_client.send_transaction(&tx).await?;

        Ok(tx_id)
    }
}
