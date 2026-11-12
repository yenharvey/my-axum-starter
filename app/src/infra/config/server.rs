use serde::Deserialize;

/// 服务器配置（`[server]`）
#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    /// 监听地址
    pub host: String,

    /// 监听端口
    pub port: u16,

    /// 单个请求的超时时间（秒），超时返回 408
    pub request_timeout_secs: u64,

    /// 是否挂载 `/docs` API 文档
    pub enable_docs: bool,
}

impl ServerConfig {
    pub(super) fn validate(&self) -> Result<(), String> {
        if self.host.is_empty() {
            return Err("host 不能为空".to_string());
        }
        if self.port == 0 {
            return Err("port 不能为 0".to_string());
        }
        if self.request_timeout_secs == 0 {
            return Err("request_timeout_secs 必须大于 0".to_string());
        }
        Ok(())
    }
}
