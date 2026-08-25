use axum::{
    http::{Request, HeaderValue},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct RequestId(pub HeaderValue);

pub async fn add_request_id<B>(
    mut req: Request<B>,
    next: Next<B>,
) -> Response {
    let request_id = match req.headers().get("x-request-id") {
        Some(val) => val.clone(),
        None => {
            let id = Uuid::new_v4().to_string();
            HeaderValue::from_str(&id).unwrap_or_else(|_| HeaderValue::from_static(""))
        }
    };

    req.extensions_mut().insert(RequestId(request_id.clone()));

    let mut response = next.run(req).await;
    response.headers_mut().insert("x-request-id", request_id);
    response
}
