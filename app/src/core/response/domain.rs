//! 错误域定义

/// 错误域
///
/// 标识错误来源的业务域，用于 `errors[].domain` 字段。
///
/// ## 为什么用 newtype 而不是枚举
///
/// 枚举是封闭集合，每新增一个业务域都必须修改这个中央文件。
/// newtype over `&'static str` 让每个模块自己定义 domain 常量，
/// 未来拆微服务时只需改一行常量值，API 契约完全不用动：
///
/// ```ignore
/// // 单体阶段
/// pub const DOMAIN: Domain = Domain("user");
///
/// // 微服务阶段，仅改这里
/// pub const DOMAIN: Domain = Domain("user.example.com");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Domain(pub &'static str);

impl Domain {
    pub const fn as_str(self) -> &'static str {
        self.0
    }

    /// 全局/通用错误（跨域 catch-all）
    pub const GLOBAL: Self = Self("global");

    /// 认证相关错误（令牌、登录凭据等）
    pub const AUTH: Self = Self("auth");

    /// 用户数据相关错误（Profile、用户名等）
    pub const USER: Self = Self("user");

    /// 请求参数验证错误（格式、范围、必填等）
    pub const VALIDATION: Self = Self("validation");

    /// 文件上传/处理错误
    pub const FILE: Self = Self("file");

    /// 速率限制/配额错误
    pub const RATE_LIMIT: Self = Self("rate_limit");
}

impl std::fmt::Display for Domain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}
