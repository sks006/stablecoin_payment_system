use orchestrator::config::Settings;
use orchestrator::api::http::server;
use orchestrator::infrastructure::db::postgres::create_pool;
use orchestrator::infrastructure::db::repositories::payment_repo::PaymentRepository;
use orchestrator::infrastructure::db::repositories::idempotency_repo::IdempotencyRepository;
use orchestrator::infrastructure::db::repositories::webhook_repo::WebhookRepository;
use orchestrator::infrastructure::webhook::sender::WebhookSender;
use orchestrator::application::webhook_dispatcher::WebhookDispatcher;
use orchestrator::jobs::webhook_retry_worker::WebhookRetryWorker;
use orchestrator::infrastructure::kms::aws_client::KmsClient;
use orchestrator::infrastructure::solana::tpu_client::TpuClient;
use orchestrator::application::transfer_service::TransactionBuilder;
use orchestrator::application::mint_service::MintService;
use orchestrator::jobs::transaction_confirmation_poller::TransactionConfirmationPoller;
use solana_client::nonblocking::rpc_client::RpcClient;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    tracing::info!("Starting stablecoin payment orchestrator...");

    // In a real environment we would load settings from a file / env
    let settings = Settings {
        database_url: std::env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://postgres:password@localhost:5432/stablecoin".to_string()),
        redis_url: std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),
        kafka_brokers: std::env::var("KAFKA_BROKERS").unwrap_or_else(|_| "localhost:9092".to_string()),
        solana_rpc_url: std::env::var("SOLANA_RPC_URL").unwrap_or_else(|_| "http://localhost:8899".to_string()),
        port: std::env::var("PORT").unwrap_or_else(|_| "8080".to_string()).parse().unwrap(),
        kms_key_id: std::env::var("KMS_KEY_ID").unwrap_or_default(),
        webhook_secret: std::env::var("WEBHOOK_SECRET").unwrap_or_else(|_| "secret".to_string()),
        webhook_url: std::env::var("WEBHOOK_URL").unwrap_or_else(|_| "http://localhost:8081/webhook".to_string()),
    };

    // Initialize database pool and repositories
    let db_pool = create_pool(&settings.database_url).await?;
    let payment_repo = Arc::new(PaymentRepository::new(db_pool.clone()));
    let idempotency_repo = Arc::new(IdempotencyRepository::new(db_pool.clone()));
    let webhook_repo = Arc::new(WebhookRepository::new(db_pool));

    // Initialize Webhook Sender and Dispatcher
    let webhook_sender = Arc::new(WebhookSender::new());
    let webhook_dispatcher = Arc::new(WebhookDispatcher::new(
        webhook_repo.clone(),
        webhook_sender.clone(),
        settings.webhook_secret.clone(),
        settings.webhook_url.clone(),
    ));

    // Initialize Solana clients
    let rpc_client = Arc::new(RpcClient::new(settings.solana_rpc_url.clone()));
    let tpu_client = Arc::new(TpuClient::new());

    // Initialize KMS client
    let kms_client = Arc::new(KmsClient::new(&settings.kms_key_id));

    // Initialize Transaction Builder with the program ID
    let program_id = "7aM25wz7W4pM3LrdHj59eSwxH517eWcKqJ7T38wFkY2c".parse::<solana_sdk::pubkey::Pubkey>()?;
    let builder = TransactionBuilder::new(program_id);

    // Initialize MintService
    let mint_service = Arc::new(MintService::new(
        idempotency_repo,
        payment_repo.clone(),
        kms_client,
        tpu_client,
        builder,
    ));

    // Set up shutdown signal and poller channels
    let (shutdown_tx, _shutdown_rx) = broadcast::channel::<()>(1);
    let poller_shutdown_rx = shutdown_tx.subscribe();
    let retry_worker_shutdown_rx = shutdown_tx.subscribe();
    let server_shutdown_rx = shutdown_tx.subscribe();

    let poller = TransactionConfirmationPoller::new(
        payment_repo,
        rpc_client,
        webhook_dispatcher,
        poller_shutdown_rx,
    );
    let poller_handle = tokio::spawn(poller.run());

    let retry_worker = WebhookRetryWorker::new(
        webhook_repo,
        webhook_sender,
        settings.webhook_secret.clone(),
        settings.webhook_url.clone(),
        retry_worker_shutdown_rx,
    );
    let retry_worker_handle = tokio::spawn(retry_worker.run());

    // Start API HTTP server in a separate task
    let server_handle = tokio::spawn(async move {
        if let Err(e) = server::start(settings.port, mint_service, server_shutdown_rx).await {
            tracing::error!("HTTP server failed: {:?}", e);
        }
    });

    // Wait for Ctrl+C or SIGTERM signal
    let shutdown_signal = async {
        let ctrl_c = async {
            tokio::signal::ctrl_c()
                .await
                .expect("failed to install Ctrl+C handler");
        };

        #[cfg(unix)]
        let terminate = async {
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                .expect("failed to install SIGTERM handler")
                .recv()
                .await;
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
            _ = ctrl_c => {},
            _ = terminate => {},
        }
    };

    shutdown_signal.await;
    tracing::info!("Shutdown signal received. Sending graceful stop to all tasks.");

    // Send shutdown signal to all tasks
    shutdown_tx.send(()).ok();

    // Wait for all tasks to finish
    let _ = poller_handle.await;
    let _ = retry_worker_handle.await;
    let _ = server_handle.await;

    tracing::info!("Orchestrator stopped cleanly.");
    Ok(())
}
