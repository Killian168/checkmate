use std::sync::Arc;

use shared::services::queue_service::QueueService;
use shared::services::user_service::UserService;

#[derive(Clone)]
pub struct AppState {
    pub user_service: Arc<UserService>,
    pub queue_service: Arc<QueueService>,
}
