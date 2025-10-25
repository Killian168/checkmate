use std::sync::Arc;

use shared::services::auth_service::AuthService;
use shared::services::game_session_service::GameSessionService;
use shared::services::websocket_service::WebSocketService;

#[derive(Clone)]
pub struct AppState {
    pub websocket_service: Arc<WebSocketService>,
    pub auth_service: Arc<AuthService>,
    pub game_session_service: Arc<GameSessionService>,
}
