use crate::AppState;
use aide::axum::routing::post_with;
use aide::axum::ApiRouter;
use std::sync::Arc;

mod api;
pub mod dto;
mod service;

pub fn routes(state: Arc<AppState>) -> ApiRouter {
    ApiRouter::new()
        .api_route(
            "/register",
            post_with(api::register_user, api::register_user_docs),
        ) 
        .with_state(state)
}
