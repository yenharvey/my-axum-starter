use serde::Deserialize;

/// CORS 跨域资源共享配置（`[cors]`）
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CorsConfig {
    /// 允许的源列表（如 `["https://example.com"]`）
    ///
    /// - `["*"]`：允许任何源（仅建议用于开发环境）
    /// - `[]`：不允许任何跨域请求
    pub allow_origins: Vec<String>,

    /// 允许的 HTTP 方法列表，`["*"]` 表示全部
    pub allow_methods: Vec<String>,

    /// 允许的请求头列表，`["*"]` 表示全部
    pub allow_headers: Vec<String>,

    /// 暴露给客户端的响应头列表，`["*"]` 表示全部
    pub expose_headers: Vec<String>,

    /// 是否允许携带凭证（Cookie、Authorization）
    pub allow_credentials: bool,

    /// 预检请求（OPTIONS）结果的缓存时间（秒）
    pub max_age_secs: u64,
}

impl CorsConfig {
    pub(super) fn validate(&self) -> Result<(), String> {
        if self.allow_methods.is_empty() {
            return Err("allow_methods 不能为空".to_string());
        }

        // 规范禁止凭证与通配符同时使用，tower-http 遇到这种组合会直接 panic
        if self.allow_credentials {
            let wildcard = |values: &[String]| values.iter().any(|value| value == "*");
            if wildcard(&self.allow_origins) {
                return Err("allow_credentials = true 时 allow_origins 不能包含 \"*\"".to_string());
            }
            if wildcard(&self.allow_methods) {
                return Err("allow_credentials = true 时 allow_methods 不能包含 \"*\"".to_string());
            }
            if wildcard(&self.allow_headers) {
                return Err("allow_credentials = true 时 allow_headers 不能包含 \"*\"".to_string());
            }
            if wildcard(&self.expose_headers) {
                return Err("allow_credentials = true 时 expose_headers 不能包含 \"*\"".to_string());
            }
        }

        Ok(())
    }
}
