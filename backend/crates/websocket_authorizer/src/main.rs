use lambda_runtime::{
    run, service_fn,
    tracing::{error, info, init_default_subscriber},
    Error, LambdaEvent,
};

use crate::auth::AuthService;
use crate::models::{AuthPolicy, WebsocketAuthorizerEvent};

mod auth;
mod models;

#[tokio::main]
async fn main() -> Result<(), Error> {
    init_default_subscriber();

    let auth_service = AuthService::new()?;

    run(service_fn(|event| authorizer_handler(event, &auth_service))).await
}

async fn authorizer_handler(
    event: LambdaEvent<WebsocketAuthorizerEvent>,
    auth_service: &AuthService,
) -> Result<AuthPolicy, Error> {
    info!("Received websocket authorizer request");

    // Only support Bearer tokens
    let token = if let Some(headers) = &event.payload.headers {
        let auth_header = headers
            .get("Authorization")
            .or_else(|| headers.get("authorization"));
        if let Some(auth_header) = auth_header {
            if auth_header.starts_with("Bearer ") {
                auth_header[7..].to_string()
            } else {
                error!("Authorization header must use Bearer scheme");
                return Ok(AuthPolicy::deny());
            }
        } else {
            error!("No Authorization header provided");
            return Ok(AuthPolicy::deny());
        }
    } else {
        error!("No headers in request");
        return Ok(AuthPolicy::deny());
    };

    // Verify JWT
    match auth_service.verify_id_token(&token).await {
        Ok(claims) => {
            let resource = event.payload.method_arn;
            let policy = AuthPolicy::allow(claims.sub, resource);
            info!("Returning Auth Policy {:?}", policy);
            let json = serde_json::to_string_pretty(&policy).unwrap_or_default();
            info!("Policy JSON: {}", json);
            Ok(policy)
        }
        Err(e) => {
            error!("JWT verification failed: {:?}", e);
            Ok(AuthPolicy::deny())
        }
    }
}
