use entity::enums::UserStatus;
use entity::user;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};

use crate::AppError;
use crate::shared::error::AuthError;
use crate::shared::password::hash_password;

/// 默认演示用户。
///
/// 种子数据固定写在代码里，避免把默认账号、邮箱、密码散落到 `.env`。
/// 密码是公开的，因此只应在本地开发和演示环境启用 `database.seed_demo`。
const DEMO_USERNAME: &str = "demo";
const DEMO_EMAIL: &str = "demo@example.com";
const DEMO_PASSWORD: &str = "demo-password";

/// 初始化演示种子数据（由 `database.seed_demo` 开启）。
///
/// 重复启动时按邮箱幂等跳过。
pub async fn seed_demo_data(db: &DatabaseConnection) -> Result<(), AppError> {
    seed_demo_user(db).await?;
    tracing::warn!(
        username = DEMO_USERNAME,
        email = DEMO_EMAIL,
        "演示用户已就绪：它的密码是公开的，不要在生产环境启用 database.seed_demo"
    );
    Ok(())
}

/// 幂等创建默认演示用户。
///
/// 如果邮箱已存在，认为种子数据已经初始化完成，不再覆盖用户已有内容。
async fn seed_demo_user(db: &DatabaseConnection) -> Result<(), AppError> {
    if user::Entity::find()
        .filter(user::Column::Email.eq(DEMO_EMAIL))
        .one(db)
        .await?
        .is_some()
    {
        return Ok(());
    }

    let password_hash = hash_password(DEMO_PASSWORD.to_string())
        .await
        .map_err(|error| AuthError::Internal(error.to_string()))?;

    user::ActiveModel {
        username: Set(DEMO_USERNAME.to_string()),
        email: Set(DEMO_EMAIL.to_string()),
        password_hash: Set(password_hash),
        status: Set(UserStatus::Active),
        ..Default::default()
    }
    .insert(db)
    .await?;

    Ok(())
}
