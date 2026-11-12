//! JSON 请求体提取器

use aide::OperationInput;
use aide::generate::GenContext;
use aide::openapi::Operation;
use axum::Json;
use axum::extract::{FromRequest, Request};
use axum::response::{IntoResponse, Response};
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use validator::Validate;

use crate::ApiResponse;
use crate::shared::error::ValidationError;
use crate::shared::response::{ApiError, Domain, ErrorDetail, Reason};

/// 带校验的 JSON 请求体提取器，用法与 `axum::Json` 相同
///
/// 与 `axum::Json` 的区别：
/// - 请求体不是合法 JSON、缺字段、类型不符或缺少 `Content-Type` 时，
///   返回统一格式的错误响应（保留 axum 给出的状态码：400 / 415 / 422 / 413）
/// - 解析成功后自动执行 `validator` 的校验规则，失败时返回 400，
///   `errors` 里每个字段一条，`location` 是字段名
///
/// ```ignore
/// #[derive(Deserialize, Validate, JsonSchema)]
/// struct CreatePost {
///     #[validate(length(min = 1, max = 100, message = "标题长度必须在 1-100 个字符之间"))]
///     title: String,
/// }
///
/// async fn create_post(ApiJson(req): ApiJson<CreatePost>) -> ApiResponse<Post> { ... }
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct ApiJson<T>(pub T);

impl<S, T> FromRequest<S> for ApiJson<T>
where
    S: Send + Sync,
    T: DeserializeOwned + Validate,
{
    type Rejection = Response;

    async fn from_request(request: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(request, state)
            .await
            .map_err(|rejection| {
                let message = rejection.body_text();
                let error = ApiError::new(rejection.status(), message.clone()).with_detail(
                    ErrorDetail::with_message(Domain::VALIDATION, Reason::InvalidFormat, message),
                );
                ApiResponse::error(error).into_response()
            })?;

        value
            .validate()
            .map_err(|errors| ValidationError::from_validator(errors).into_response())?;

        Ok(Self(value))
    }
}

impl<T: JsonSchema> OperationInput for ApiJson<T> {
    fn operation_input(ctx: &mut GenContext, operation: &mut Operation) {
        Json::<T>::operation_input(ctx, operation);
    }
}
