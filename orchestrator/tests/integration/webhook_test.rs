#[cfg(test)]
mod tests {
    use orchestrator::infrastructure::webhook::signature::calculate_signature;
    use orchestrator::infrastructure::webhook::sender::WebhookSender;
    use orchestrator::infrastructure::db::repositories::webhook_repo::WebhookRepository;
    use orchestrator::application::webhook_dispatcher::WebhookDispatcher;
    use orchestrator::jobs::webhook_retry_worker::WebhookRetryWorker;
    use orchestrator::domain::webhook_event::WebhookStatus;
    use orchestrator::domain::payment::{Payment, PaymentStatus};

    use axum::{
        routing::post,
        Router,
        Json,
        http::HeaderMap,
    };
    use std::net::SocketAddr;
    use std::sync::Arc;
    use tokio::sync::mpsc;
    use serde_json::Value;
    use uuid::Uuid;
    use chrono::Utc;

    struct ReceivedWebhook {
        headers: HeaderMap,
        payload: Value,
    }

    async fn spawn_mock_merchant() -> (SocketAddr, mpsc::Receiver<ReceivedWebhook>) {
        let (tx, rx) = mpsc::channel(10);
        
        let app = Router::new().route("/webhook", post(move |headers: HeaderMap, Json(payload): Json<Value>| {
            let tx = tx.clone();
            async move {
                let _ = tx.send(ReceivedWebhook { headers, payload }).await;
                axum::http::StatusCode::OK
            }
        }));

        let addr = SocketAddr::from(([127, 0, 0, 1], 0));
        let server = axum::Server::bind(&addr).serve(app.into_make_service());
        let local_addr = server.local_addr();
        
        tokio::spawn(async move {
            server.await.unwrap();
        });

        (local_addr, rx)
    }

    async fn get_db_pool() -> Option<sqlx::PgPool> {
        let db_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:password@localhost:5432/stablecoin".to_string());
        
        sqlx::PgPool::connect(&db_url).await.ok()
    }

    #[test]
    fn test_signature_verification() {
        let secret = "my_secret_key";
        let timestamp = 1629900000;
        let payload = r#"{"event":"test"}"#;
        
        let sig1 = calculate_signature(secret, timestamp, payload);
        let sig2 = calculate_signature(secret, timestamp, payload);
        
        assert_eq!(sig1, sig2);
        assert!(!sig1.is_empty());
    }

    #[tokio::test]
    async fn test_webhook_sender_success() {
        let (addr, mut rx) = spawn_mock_merchant().await;
        let sender = WebhookSender::new();
        
        let url = format!("http://{}/webhook", addr);
        let secret = "secret_key";
        let event_id = "event-123";
        let timestamp = 1234567890;
        let payload = serde_json::json!({ "foo": "bar" });
        
        let res = sender.send(&url, secret, event_id, timestamp, &payload).await;
        assert!(res.is_ok());
        
        let received = rx.recv().await.expect("No webhook received");
        
        let sig_header = received.headers.get("X-Webhook-Signature").unwrap().to_str().unwrap();
        let ts_header = received.headers.get("X-Webhook-Timestamp").unwrap().to_str().unwrap();
        let id_header = received.headers.get("X-Webhook-Event-Id").unwrap().to_str().unwrap();
        
        assert_eq!(ts_header, "1234567890");
        assert_eq!(id_header, "event-123");
        
        let expected_sig = calculate_signature(secret, 1234567890, &payload.to_string());
        assert_eq!(sig_header, expected_sig);
        assert_eq!(received.payload["foo"], "bar");
    }

    #[tokio::test]
    async fn test_webhook_dispatcher_and_retry_worker() {
        let db_pool = match get_db_pool().await {
            Some(pool) => pool,
            None => {
                println!("Skipping database-dependent webhook tests: Postgres connection failed");
                return;
            }
        };

        // Initialize dependencies
        let webhook_repo = Arc::new(WebhookRepository::new(db_pool.clone()));
        let webhook_sender = Arc::new(WebhookSender::new());
        let (addr, mut merchant_rx) = spawn_mock_merchant().await;
        let webhook_url = format!("http://{}/webhook", addr);
        let secret = "test_secret";

        let dispatcher = WebhookDispatcher::new(
            webhook_repo.clone(),
            webhook_sender.clone(),
            secret.to_string(),
            webhook_url.clone(),
        );

        // Clean up database tables for clean testing
        sqlx::query("DELETE FROM webhook_events").execute(&db_pool).await.unwrap();
        sqlx::query("DELETE FROM payments").execute(&db_pool).await.unwrap();

        // Save a mock payment
        let payment_id = Uuid::new_v4();
        sqlx::query("INSERT INTO payments (id, idempotency_key, amount, sender, recipient, status, signature) VALUES ($1, $2, $3, $4, $5, $6, $7)")
            .bind(payment_id)
            .bind("key_123")
            .bind(100_i64)
            .bind("sender_wallet")
            .bind("recipient_wallet")
            .bind("confirmed")
            .bind("signature_hash")
            .execute(&db_pool)
            .await
            .unwrap();

        let payment = Payment {
            id: payment_id,
            idempotency_key: "key_123".to_string(),
            amount: 100,
            sender: "sender_wallet".to_string(),
            recipient: "recipient_wallet".to_string(),
            status: PaymentStatus::Confirmed,
            signature: Some("signature_hash".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // Dispatch webhook
        dispatcher.dispatch(&payment, "payment.confirmed").await.unwrap();

        // Verify mock merchant received it immediately
        let received = merchant_rx.recv().await.expect("No immediate webhook received");
        assert_eq!(received.payload["payment_id"], payment_id.to_string());

        // Verify status in DB is Delivered
        let events = webhook_repo.get_pending_or_failed_events().await.unwrap();
        assert!(events.is_empty(), "All events should have been delivered");

        // Now test retry worker with a failing URL
        let failing_dispatcher = WebhookDispatcher::new(
            webhook_repo.clone(),
            webhook_sender.clone(),
            secret.to_string(),
            "http://127.0.0.1:1/webhook".to_string(), // port 1 should fail to connect
        );

        failing_dispatcher.dispatch(&payment, "payment.failed_retry").await.unwrap();

        // Verify it failed and next_retry_at is populated
        let events = webhook_repo.get_pending_or_failed_events().await.unwrap();
        assert_eq!(events.len(), 1);
        let event = &events[0];
        assert_eq!(event.status, WebhookStatus::Failed);
        assert_eq!(event.retry_count, 1);
        assert!(event.next_retry_at.is_some());

        // Manually update next_retry_at to the past so worker processes it immediately
        sqlx::query("UPDATE webhook_events SET next_retry_at = NOW() - INTERVAL '1 minute'")
            .execute(&db_pool)
            .await
            .unwrap();

        // Run retry worker with shutdown broadcast
        let (shutdown_tx, shutdown_rx) = tokio::sync::broadcast::channel(1);
        let worker = WebhookRetryWorker::new(
            webhook_repo.clone(),
            webhook_sender.clone(),
            secret.to_string(),
            "http://127.0.0.1:1/webhook".to_string(),
            shutdown_rx,
        );

        let worker_handle = tokio::spawn(worker.run());

        // Wait a second for worker to run once
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // Shutdown retry worker
        shutdown_tx.send(()).unwrap();
        let _ = worker_handle.await;

        // Verify retry count increased
        let events = webhook_repo.get_pending_or_failed_events().await.unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].retry_count, 2);
    }
}
