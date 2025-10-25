use aws_lambda_events::apigw::ApiGatewayWebsocketProxyRequest;
use lambda_runtime::Error;
use shared::services::auth_service::{AuthService, AuthServiceTrait};
use std::sync::Arc;

#[derive(Clone)]
pub struct WebSocketAuth {
    auth_service: Arc<AuthService>,
}

impl WebSocketAuth {
    pub fn new(auth_service: Arc<AuthService>) -> Self {
        Self { auth_service }
    }

    /// Authenticate a websocket connection using JWT token from query parameters
    /// Returns the authenticated user ID if successful
    pub fn authenticate_connection(
        &self,
        event: &ApiGatewayWebsocketProxyRequest,
    ) -> Result<String, Error> {
        // Extract token from query parameters
        let token = event
            .query_string_parameters
            .first("token")
            .ok_or_else(|| Error::from("Missing token query parameter"))?;

        // Verify JWT token and extract user ID
        match self.auth_service.extract_user_id_from_token(token) {
            Ok(user_id) => Ok(user_id),
            Err(_) => Err(Error::from("Invalid or expired token")),
        }
    }
}
