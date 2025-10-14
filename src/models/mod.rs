use std::sync::Arc;

use crate::services::auth_service::AuthService;
use crate::services::user_service::UserService;

pub mod auth;
pub mod user;

#[derive(Clone)]
pub struct AppState {
    pub auth_service: Arc<AuthService>,
    pub user_service: Arc<UserService>,
}
