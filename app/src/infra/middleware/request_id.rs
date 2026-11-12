use axum::extract::Request;
use axum::http::HeaderValue;
use axum::middleware::Next;
use axum::response::Response;
use uuid::Uuid;

pub async fn request_id_middleware(mut request: Request, next: Next) -> Response {
    let request_id = Uuid::new_v4().to_string();

    // 将请求ID添加到请求头中
    if let Ok(header_value) = HeaderValue::from_str(&request_id) {
        request.headers_mut().insert("x-request-id", header_value);
    }

    let mut response = next.run(request).await;

    // 将请求ID添加到响应头中
    if let Ok(header_value) = HeaderValue::from_str(&request_id) {
        response.headers_mut().insert("x-request-id", header_value);
    }

    response
}
