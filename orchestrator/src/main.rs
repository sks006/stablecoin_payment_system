use orchestrator::config::Settings;
use orchestrator::api::http::server;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().or_else(|_| EnvFilter::try_new("info")))
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
    };

    server::start(&settings).await?;

    Ok(())
}
