use std::sync::Arc;

use crate::services::auth_service::AuthService;
use crate::services::matchmaking_service::MatchmakingService;
use crate::services::user_service::UserService;

/// Application state specific to the API package
/// This contains the actual service instances with their concrete types
#[derive(Clone)]
pub struct AppState {
    pub auth_service: Arc<AuthService>,
    pub user_service: Arc<UserService>,
    pub matchmaking_service: Arc<MatchmakingService>,
}
