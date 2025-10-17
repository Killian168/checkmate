pub mod auth;
pub mod matchmaking;
pub mod user;

use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub auth_service: Arc<dyn std::any::Any + Send + Sync>,
    pub user_service: Arc<dyn std::any::Any + Send + Sync>,
    pub matchmaking_service: Arc<dyn std::any::Any + Send + Sync>,
}
