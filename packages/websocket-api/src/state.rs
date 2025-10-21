use std::sync::Arc;

use shared::services::websocket_service::WebSocketService;

#[derive(Clone)]
pub struct AppState {
    pub websocket_service: Arc<WebSocketService>,
}
