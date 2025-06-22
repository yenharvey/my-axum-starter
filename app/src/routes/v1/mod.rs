use crate::{auth, AppState};
use aide::axum::ApiRouter;
use std::sync::Arc;

pub fn routes(state: Arc<AppState>) -> ApiRouter {
    ApiRouter::new()
        .nest_api_service("/auth", auth::routes(state.clone()))
        .with_state(state)
}
