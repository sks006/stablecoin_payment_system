use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub database_url: String,
    pub redis_url: String,
    pub kafka_brokers: String,
    pub solana_rpc_url: String,
    pub port: u16,
    pub kms_key_id: String,
    pub webhook_secret: String,
}
