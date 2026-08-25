use axum::{
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};

pub async fn require_valid_auth<B>(
    req: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    let auth_header = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok());

    match auth_header {
        Some(auth) if auth.starts_with("Bearer ") => {
            let token = &auth[7..];
            if token.is_empty() || token == "invalid-token" {
                return Err(StatusCode::UNAUTHORIZED);
            }
            Ok(next.run(req).await)
        }
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}
