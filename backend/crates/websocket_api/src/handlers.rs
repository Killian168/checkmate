use lambda_runtime::Error;
use serde_json;
use tracing::{error, info};

use crate::connections::{get_user_id_by_connection, remove_connection, store_connection};
use crate::models::{Connection, JoinQueueMessage, LeaveQueueMessage, ResponseMessage};
use crate::queue::{join_queue, leave_queue};
use shared::auth::extract_claims;

pub async fn handle_connect(
    request: &aws_lambda_events::event::apigw::ApiGatewayWebsocketProxyRequest,
    state: &crate::AppState,
) -> Result<(), Error> {
    let request_context = &request.request_context;
    let connection_id = request_context.connection_id.as_deref().unwrap_or("");

    // Extract and validate JWT
    let auth_header = request
        .headers
        .get("authorization")
        .or_else(|| request.headers.get("Authorization"))
        .and_then(|v| v.to_str().ok());

    // Print all headers
    for (key, value) in &request.headers {
        info!("Header: {} = {:?}", key, value);
    }

    let user_id = if let Some(header) = auth_header {
        if header.starts_with("Bearer ") {
            let token = &header[7..];
            let claims = extract_claims(token)?;
            info!(
                "JWT validated, user_id: {} for connection {}",
                claims.sub, connection_id
            );
            claims.sub
        } else {
            error!(
                "Invalid auth header format for connection {}",
                connection_id
            );
            return Err("Invalid auth header format".into());
        }
    } else {
        error!(
            "Missing Authorization header for connection {}",
            connection_id
        );
        return Err("Missing Authorization header".into());
    };

    if user_id.is_empty() {
        error!("Invalid user_id for connection {}", connection_id);
        return Err("Invalid user_id".into());
    }

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs()
        .to_string();

    let connection = Connection {
        connection_id: connection_id.to_string(),
        user_id: user_id.to_string(),
        connected_at: now,
    };

    info!(
        "Storing connection for user {} with connection_id {}",
        user_id, connection_id
    );
    store_connection(state, &connection).await?;
    info!(
        "User {} connected with connection_id {}",
        user_id, connection_id
    );
    Ok(())
}

pub async fn handle_disconnect(
    request_context: &aws_lambda_events::event::apigw::ApiGatewayWebsocketProxyRequestContext,
    state: &crate::AppState,
) -> Result<(), Error> {
    let connection_id = request_context.connection_id.as_deref().unwrap_or("");
    info!("Removing connection for connection_id {}", connection_id);
    remove_connection(state, connection_id).await?;
    info!("Connection {} disconnected", connection_id);
    Ok(())
}

pub async fn handle_join_queue(
    request_context: &aws_lambda_events::event::apigw::ApiGatewayWebsocketProxyRequestContext,
    body: &str,
    state: &crate::AppState,
) -> Result<(), Error> {
    let connection_id = request_context.connection_id.as_deref().unwrap_or("");
    info!("Retrieving user_id for connection_id {}", connection_id);
    let user_id = get_user_id_by_connection(state, connection_id)
        .await?
        .ok_or("Not connected")?;
    info!("User {} is connected", user_id);

    info!("Parsing join queue message from body: {}", body);
    let join_msg: JoinQueueMessage = serde_json::from_str(body)?;
    info!(
        "Joining queue for user {} with time_control {} min_rating {:?} max_rating {:?}",
        user_id, join_msg.time_control, join_msg.min_rating, join_msg.max_rating
    );
    join_queue(state, &user_id, &join_msg).await?;
    info!("Successfully joined queue for user {}", user_id);

    info!(
        "Sending success response for join_queue to connection {}",
        connection_id
    );
    send_response(
        request_context,
        &ResponseMessage {
            status: "success".to_string(),
            message: "Joined queue".to_string(),
        },
        state,
    )
    .await?;
    info!(
        "Join queue response sent successfully to connection {}",
        connection_id
    );
    Ok(())
}

pub async fn handle_leave_queue(
    request_context: &aws_lambda_events::event::apigw::ApiGatewayWebsocketProxyRequestContext,
    body: &str,
    state: &crate::AppState,
) -> Result<(), Error> {
    let connection_id = request_context.connection_id.as_deref().unwrap_or("");
    info!("Retrieving user_id for connection_id {}", connection_id);
    let user_id = get_user_id_by_connection(state, connection_id)
        .await?
        .ok_or("Not connected")?;
    info!("User {} is connected", user_id);

    info!("Parsing leave queue message from body: {}", body);
    let leave_msg: LeaveQueueMessage = serde_json::from_str(body)?;
    info!(
        "Leaving queue for user {} with time_control {}",
        user_id, leave_msg.time_control
    );
    leave_queue(state, &user_id, &leave_msg.time_control).await?;
    info!("Successfully left queue for user {}", user_id);

    info!(
        "Sending success response for leave_queue to connection {}",
        connection_id
    );
    send_response(
        request_context,
        &ResponseMessage {
            status: "success".to_string(),
            message: "Left queue".to_string(),
        },
        state,
    )
    .await?;
    info!(
        "Leave queue response sent successfully to connection {}",
        connection_id
    );
    Ok(())
}

pub async fn handle_default(
    request_context: &aws_lambda_events::event::apigw::ApiGatewayWebsocketProxyRequestContext,
    state: &crate::AppState,
) -> Result<(), Error> {
    let connection_id = request_context.connection_id.as_deref().unwrap_or("");
    info!(
        "Sending unknown action error response to connection {}",
        connection_id
    );
    send_response(
        request_context,
        &ResponseMessage {
            status: "error".to_string(),
            message: "Unknown action".to_string(),
        },
        state,
    )
    .await?;
    info!(
        "Unknown action error response sent successfully to connection {}",
        connection_id
    );
    Ok(())
}

async fn send_response(
    request_context: &aws_lambda_events::event::apigw::ApiGatewayWebsocketProxyRequestContext,
    response: &ResponseMessage,
    state: &crate::AppState,
) -> Result<(), Error> {
    let connection_id = request_context.connection_id.as_deref().unwrap_or("");

    info!(
        "Preparing to send response to connection {}: status={}, message={}",
        connection_id, response.status, response.message
    );
    let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
    let endpoint_url = &state.websocket_api_endpoint;
    info!("Using API Gateway endpoint: {}", endpoint_url);
    let api_config = aws_sdk_apigatewaymanagement::config::Builder::from(&config)
        .endpoint_url(endpoint_url)
        .build();
    let client = aws_sdk_apigatewaymanagement::Client::from_conf(api_config);

    let data = serde_json::to_string(response)?;
    info!("Sending data: {}", data);
    client
        .post_to_connection()
        .connection_id(connection_id)
        .data(aws_sdk_apigatewaymanagement::primitives::Blob::new(data))
        .send()
        .await?;
    info!("Successfully sent response to connection {}", connection_id);
    Ok(())
}
