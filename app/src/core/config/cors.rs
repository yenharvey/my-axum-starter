use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::section::ConfigSection;

/// CORS 跨域资源共享配置
///
/// 用于控制浏览器跨域请求的安全政策。包括允许的源、请求头、
/// 凭证共享等配置。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CorsConfig {
    /// 允许的源列表（如：["http://localhost:3000", "https://example.com"]）
    /// 使用 "*" 表示允许任何源（不安全，不推荐用于生产）
    pub allow_origins: Vec<String>,

    /// 允许的 HTTP 方法列表（如：["GET", "POST", "PUT", "DELETE"]）
    /// （默认：["GET", "POST", "PUT", "DELETE", "OPTIONS", "HEAD"]）
    pub allow_methods: Vec<String>,

    /// 允许的请求头列表（如：["Authorization", "Content-Type"]）
    /// （默认：["Authorization", "Content-Type", "Accept", "X-Request-ID"]）
    pub allow_headers: Vec<String>,

    /// 是否允许凭证（Cookie、Authorization）跨域传送
    /// （默认：false）
    pub allow_credentials: bool,

    /// 暴露给客户端的响应头列表
    /// （默认：["Content-Type", "X-Total-Count"]）
    pub expose_headers: Vec<String>,

    /// 预检请求（OPTIONS）的缓存时间，单位秒
    /// （默认：3600）
    pub max_age: u64,
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            allow_origins: vec!["*".to_string()],
            allow_methods: vec![
                "GET".to_string(),
                "POST".to_string(),
                "PUT".to_string(),
                "DELETE".to_string(),
                "OPTIONS".to_string(),
                "HEAD".to_string(),
            ],
            allow_headers: vec![
                "Authorization".to_string(),
                "Content-Type".to_string(),
                "Accept".to_string(),
                "X-Request-ID".to_string(),
            ],
            allow_credentials: false,
            expose_headers: vec!["Content-Type".to_string(), "X-Total-Count".to_string()],
            max_age: 3600,
        }
    }
}

impl ConfigSection for CorsConfig {
    fn section_name(&self) -> &str {
        "cors"
    }

    fn load_from_value(&mut self, value: &Value) -> Result<(), String> {
        if let Some(obj) = value.as_object() {
            if let Some(origins) = obj.get("allow_origins").and_then(|v| v.as_array()) {
                self.allow_origins = origins
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();
            }
            if let Some(methods) = obj.get("allow_methods").and_then(|v| v.as_array()) {
                self.allow_methods = methods
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();
            }
            if let Some(headers) = obj.get("allow_headers").and_then(|v| v.as_array()) {
                self.allow_headers = headers
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();
            }
            if let Some(credentials) = obj.get("allow_credentials").and_then(|v| v.as_bool()) {
                self.allow_credentials = credentials;
            }
            if let Some(expose) = obj.get("expose_headers").and_then(|v| v.as_array()) {
                self.expose_headers = expose
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();
            }
            if let Some(age) = obj.get("max_age").and_then(|v| v.as_u64()) {
                self.max_age = age;
            }
        }
        Ok(())
    }

    fn validate(&self) -> Result<(), String> {
        if self.allow_origins.is_empty() {
            return Err("CORS 允许源列表不能为空".to_string());
        }
        if self.allow_methods.is_empty() {
            return Err("CORS 允许方法列表不能为空".to_string());
        }
        if self.allow_headers.is_empty() {
            return Err("CORS 允许请求头列表不能为空".to_string());
        }
        // 当允许凭证时，不能使用通配符方法
        if self.allow_credentials && self.allow_methods.contains(&"*".to_string()) {
            return Err(
                "Invalid CORS configuration: Cannot combine `Access-Control-Allow-Credentials: true` with `Access-Control-Allow-Methods: *`"
                    .to_string(),
            );
        }
        Ok(())
    }
}
