use entity::enums::UserStatus;
use entity::user;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use validator::Validate;

/// 用户列表项
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UserListItem {
    /// 用户ID
    pub id: i32,

    /// 用户名
    pub username: String,

    /// 邮箱
    pub email: String,

    /// 用户状态
    pub status: UserStatus,
}

impl From<user::Model> for UserListItem {
    fn from(model: user::Model) -> Self {
        Self {
            id: model.id,
            username: model.username,
            email: model.email,
            status: model.status,
        }
    }
}

/// 用户注册请求
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Validate)]
pub struct RegisterRequest {
    /// 用户名（3-20字符）
    #[validate(length(min = 3, max = 20, message = "用户名长度必须在 3-20 个字符之间"))]
    pub username: String,

    /// 邮箱地址
    #[validate(email(message = "邮箱格式无效"))]
    pub email: String,

    /// 密码（8字符以上）
    #[validate(length(min = 8, message = "密码长度至少 8 个字符"))]
    pub password: String,

    /// 确认密码
    #[validate(must_match(other = "password", message = "两次输入的密码不一致"))]
    pub password_confirm: String,
}

/// 用户注册响应
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RegisterResponse {
    /// 用户ID
    pub id: i32,

    /// 用户名
    pub username: String,

    /// 邮箱
    pub email: String,
}

/// 用户登录请求
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Validate)]
pub struct LoginRequest {
    /// 用户名或邮箱
    #[validate(length(min = 1, message = "用户名或邮箱不能为空"))]
    pub username_or_email: String,

    /// 密码
    #[validate(length(min = 1, message = "密码不能为空"))]
    pub password: String,
}

/// 用户登录响应
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LoginResponse {
    /// 用户ID
    pub id: i32,

    /// 用户名
    pub username: String,

    /// 邮箱
    pub email: String,

    /// JWT Token
    pub token: String,

    /// Token 过期时间（秒）
    pub expires_in: i64,
}
