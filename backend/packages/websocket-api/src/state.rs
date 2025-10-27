use std::sync::Arc;

use shared::services::game_session_service::GameSessionService;
use shared::services::queue_service::QueueService;
use shared::services::user_service::UserService;
use shared::services::websocket_service::WebSocketService;

#[derive(Clone)]
pub struct AppState {
    pub websocket_service: Arc<WebSocketService>,
    pub user_service: Arc<UserService>,
    pub queue_service: Arc<QueueService>,
    pub game_session_service: Arc<GameSessionService>,
}
