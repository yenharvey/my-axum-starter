use askama::Template;
use axum::{
    extract::Request,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use chrono::Utc;

#[derive(Template)]
#[template(path = "404.html")]
pub struct NotFoundTemplate {
    pub request_path: Option<String>,
    pub timestamp: Option<String>,
    pub request_id: Option<String>,
}

impl NotFoundTemplate {
    pub fn new(request_path: String, request_id: Option<String>) -> Self {
        let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();

        Self {
            request_path: Some(request_path),
            timestamp: Some(timestamp),
            request_id,
        }
    }
}

// 404 错误处理器
pub async fn handle_404(request: Request) -> Response {
    let path = request.uri().path().to_string();

    let headers = request.headers();

    let request_id = headers
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    if let Some(ref req_id) = request_id {
        tracing::debug!("404 page rendering with request-id: {}", req_id);
    } else {
        tracing::debug!(
            "404 page rendering without request-id, available headers: {:?}",
            headers.keys().collect::<Vec<_>>()
        );
    }

    let template = NotFoundTemplate::new(path, request_id);

    match template.render() {
        Ok(html) => (StatusCode::NOT_FOUND, Html(html)).into_response(),
        Err(err) => {
            tracing::error!("Failed to render 404 template: {}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response()
        }
    }
}
