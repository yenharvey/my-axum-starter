use sea_orm::DatabaseConnection;

use crate::{shared::FromState, AppError, AppState};



pub struct AuthService {
    db: DatabaseConnection,
}

impl FromState for AuthService {
    fn from_state(app: &AppState) -> Self {
        Self { db: app.db.clone() }
    }
}

impl AuthService {
    pub async fn register_user(&self, user: &str) -> Result<String, AppError> {
        Ok(user.to_string())
    }
}
