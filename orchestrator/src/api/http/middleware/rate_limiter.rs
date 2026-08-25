use axum::{
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use std::sync::Mutex;
use std::collections::HashMap;
use std::time::Instant;

struct TokenBucket {
    tokens: f64,
    last_update: Instant,
}

impl TokenBucket {
    fn new(max_tokens: f64) -> Self {
        Self {
            tokens: max_tokens,
            last_update: Instant::now(),
        }
    }

    fn take(&mut self, max_tokens: f64, refill_rate: f64) -> bool {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_update).as_secs_f64();
        self.tokens = (self.tokens + elapsed * refill_rate).min(max_tokens);
        self.last_update = now;

        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }
}

lazy_static::lazy_static! {
    static ref BUCKETS: Mutex<HashMap<String, TokenBucket>> = Mutex::new(HashMap::new());
}

pub async fn rate_limit<B>(
    req: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    let client_ip = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("127.0.0.1")
        .to_string();

    let max_tokens = 100.0;
    let refill_rate = 10.0; // 10 tokens per second

    let allowed = {
        let mut buckets = BUCKETS.lock().unwrap();
        let bucket = buckets
            .entry(client_ip)
            .or_insert_with(|| TokenBucket::new(max_tokens));
        bucket.take(max_tokens, refill_rate)
    };

    if !allowed {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    Ok(next.run(req).await)
}
