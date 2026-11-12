use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, DbErr, EntityTrait,
    PaginatorTrait, QueryFilter, QueryOrder, Set, SqlErr,
};
use tracing::instrument;
use validator::ValidateEmail;

use crate::AppState;
use crate::shared::error::AuthError;
use crate::shared::pagination::Pagination;
use crate::shared::{FromState, jwt::JwtService, password};
use entity::enums::UserStatus;
use entity::user;

use super::dto::{LoginRequest, LoginResponse, RegisterRequest, RegisterResponse, UserListItem};

/// 用户名长度范围（按字符计）
const USERNAME_LEN: std::ops::RangeInclusive<usize> = 3..=20;
/// 密码最小长度
const MIN_PASSWORD_LEN: usize = 8;

/// 用户服务
///
/// 处理用户注册、登录等业务逻辑
pub struct UserService {
    db: DatabaseConnection,
    jwt_service: JwtService,
}

impl FromState for UserService {
    fn from_state(app: &AppState) -> Self {
        Self {
            db: app.db.clone(),
            jwt_service: app.jwt_service.clone(),
        }
    }
}

impl UserService {
    /// 分页查询用户列表，按创建时间倒序。返回当前页的数据和总行数。
    #[instrument(skip(self))]
    pub async fn list_users(
        &self,
        pagination: Pagination,
    ) -> Result<(Vec<UserListItem>, u64), AuthError> {
        let paginator = user::Entity::find()
            .order_by_desc(user::Column::CreatedAt)
            // 创建时间相同时用主键保证翻页顺序稳定
            .order_by_desc(user::Column::Id)
            .paginate(&self.db, pagination.page_size);
        let total = paginator.num_items().await.map_err(db_error)?;
        let rows = paginator
            .fetch_page(pagination.zero_based_page())
            .await
            .map_err(db_error)?;

        Ok((rows.into_iter().map(UserListItem::from).collect(), total))
    }

    /// 用户注册
    ///
    /// 1. 校验用户名、邮箱、密码
    /// 2. 检查用户名和邮箱是否已被注册
    /// 3. 使用 Argon2 哈希密码并创建用户
    #[instrument(skip_all, fields(username = %req.username))]
    pub async fn register(&self, req: RegisterRequest) -> Result<RegisterResponse, AuthError> {
        // 按字符数（而非字节数）计算长度，否则中文等多字节用户名会被误判
        if !USERNAME_LEN.contains(&req.username.chars().count()) {
            return Err(AuthError::InvalidUsername);
        }
        if !req.email.validate_email() {
            return Err(AuthError::InvalidEmail);
        }
        if req.password.chars().count() < MIN_PASSWORD_LEN {
            return Err(AuthError::PasswordTooShort);
        }
        if req.password != req.password_confirm {
            return Err(AuthError::PasswordMismatch);
        }

        let existing = user::Entity::find()
            .filter(
                Condition::any()
                    .add(user::Column::Username.eq(&req.username))
                    .add(user::Column::Email.eq(&req.email)),
            )
            .one(&self.db)
            .await
            .map_err(db_error)?;
        if existing.is_some() {
            return Err(AuthError::UserAlreadyExists);
        }

        let password_hash = password::hash_password(req.password)
            .await
            .map_err(|e| AuthError::Internal(e.to_string()))?;

        let user_model = user::ActiveModel {
            username: Set(req.username),
            email: Set(req.email),
            password_hash: Set(password_hash),
            status: Set(UserStatus::Active),
            ..Default::default()
        }
        .insert(&self.db)
        .await
        .map_err(|e| match e.sql_err() {
            // 并发注册时前面的存在性检查可能同时通过，由唯一索引兜底
            Some(SqlErr::UniqueConstraintViolation(_)) => AuthError::UserAlreadyExists,
            _ => db_error(e),
        })?;

        Ok(RegisterResponse {
            id: user_model.id,
            username: user_model.username,
            email: user_model.email,
        })
    }

    /// 用户登录
    ///
    /// 1. 根据用户名或邮箱查询用户并验证密码
    /// 2. 检查用户状态（必须是激活状态）
    /// 3. 签发 JWT 令牌
    ///
    /// “用户不存在”和“密码错误”返回同一个错误，避免泄露账号是否存在。
    #[instrument(skip_all, fields(username_or_email = %req.username_or_email))]
    pub async fn login(&self, req: LoginRequest) -> Result<LoginResponse, AuthError> {
        let user_model = user::Entity::find()
            .filter(
                Condition::any()
                    .add(user::Column::Username.eq(&req.username_or_email))
                    .add(user::Column::Email.eq(&req.username_or_email)),
            )
            .one(&self.db)
            .await
            .map_err(db_error)?
            .ok_or(AuthError::InvalidCredentials)?;

        let password_valid =
            password::verify_password(req.password, user_model.password_hash.clone())
                .await
                .map_err(|e| AuthError::Internal(e.to_string()))?;
        if !password_valid {
            return Err(AuthError::InvalidCredentials);
        }

        // 密码验证通过之后才透露账号状态
        if user_model.status != UserStatus::Active {
            return Err(AuthError::UserInactive);
        }

        let token = self
            .jwt_service
            .generate_token(user_model.id)
            .map_err(|e| AuthError::Internal(e.to_string()))?;

        Ok(LoginResponse {
            id: user_model.id,
            username: user_model.username,
            email: user_model.email,
            token,
            expires_in: self.jwt_service.ttl_secs(),
        })
    }

    /// 根据用户 ID 获取用户信息
    #[instrument(skip(self))]
    pub async fn get_user(&self, user_id: i32) -> Result<RegisterResponse, AuthError> {
        let user_model = user::Entity::find_by_id(user_id)
            .one(&self.db)
            .await
            .map_err(db_error)?
            .ok_or(AuthError::UserNotFound)?;

        Ok(RegisterResponse {
            id: user_model.id,
            username: user_model.username,
            email: user_model.email,
        })
    }
}

/// 数据库错误统一转为内部错误（细节只进日志）
fn db_error(e: DbErr) -> AuthError {
    AuthError::Internal(format!("数据库操作失败: {e}"))
}
