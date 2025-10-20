use std::sync::Arc;

use shared::services::auth_service::AuthService;
use shared::services::queue_service::QueueService;
use shared::services::user_service::UserService;

#[derive(Clone)]
pub struct AppState {
    pub auth_service: Arc<AuthService>,
    pub user_service: Arc<UserService>,
    pub queue_service: Arc<QueueService>,
}
