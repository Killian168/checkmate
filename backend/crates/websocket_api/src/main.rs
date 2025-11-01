use aws_lambda_events::event::apigw::{ApiGatewayProxyResponse, ApiGatewayWebsocketProxyRequest};
use lambda_runtime::{
    run, service_fn,
    tracing::{error, info, init_default_subscriber, warn},
    Error, LambdaEvent,
};

use websocket_api::handlers::{
    handle_connect, handle_default, handle_disconnect, handle_join_queue, handle_leave_queue,
};
use websocket_api::AppState;

#[tokio::main]
async fn main() -> Result<(), Error> {
    init_default_subscriber();

    let state = AppState::new().await;
    run(service_fn(|event| handler(event, state.clone()))).await
}

async fn handler(
    event: LambdaEvent<ApiGatewayWebsocketProxyRequest>,
    state: AppState,
) -> Result<ApiGatewayProxyResponse, Error> {
    info!("Received websocket event");
    let request = event.payload;
    info!("Got request");
    let request_context = &request.request_context;
    info!("Got request context");
    let connection_id = request_context
        .connection_id
        .as_deref()
        .unwrap_or("unknown");
    info!("Got connection id");
    let route = request_context.route_key.as_deref().unwrap_or("$default");
    info!("Got route");

    info!(
        "Received websocket event: route={}, connection_id={}, body={:?}",
        route, connection_id, request.body
    );

    let result = match route {
        "$connect" => {
            info!("Processing $connect for connection {}", connection_id);
            let res = handle_connect(&request, &state).await;
            if res.is_ok() {
                info!(
                    "$connect handled successfully for connection {}",
                    connection_id
                );
            } else {
                error!(
                    "$connect failed for connection {}: {:?}",
                    connection_id, res
                );
            }
            res
        }
        "$disconnect" => {
            info!("Processing $disconnect for connection {}", connection_id);
            let res = handle_disconnect(request_context, &state).await;
            if res.is_ok() {
                info!(
                    "$disconnect handled successfully for connection {}",
                    connection_id
                );
            } else {
                error!(
                    "$disconnect failed for connection {}: {:?}",
                    connection_id, res
                );
            }
            res
        }
        "join_queue" => {
            if let Some(body) = &request.body {
                info!(
                    "Processing join_queue for connection {} with body: {}",
                    connection_id, body
                );
                let res = handle_join_queue(request_context, body, &state).await;
                if res.is_ok() {
                    info!(
                        "join_queue handled successfully for connection {}",
                        connection_id
                    );
                } else {
                    error!(
                        "join_queue failed for connection {}: {:?}",
                        connection_id, res
                    );
                }
                res
            } else {
                warn!(
                    "No body provided for join_queue for connection {}",
                    connection_id
                );
                Ok(())
            }
        }
        "leave_queue" => {
            if let Some(body) = &request.body {
                info!(
                    "Processing leave_queue for connection {} with body: {}",
                    connection_id, body
                );
                let res = handle_leave_queue(request_context, body, &state).await;
                if res.is_ok() {
                    info!(
                        "leave_queue handled successfully for connection {}",
                        connection_id
                    );
                } else {
                    error!(
                        "leave_queue failed for connection {}: {:?}",
                        connection_id, res
                    );
                }
                res
            } else {
                warn!(
                    "No body provided for leave_queue for connection {}",
                    connection_id
                );
                Ok(())
            }
        }
        _ => {
            info!(
                "Processing default route {} for connection {}",
                route, connection_id
            );
            let res = handle_default(request_context, &state).await;
            if res.is_ok() {
                info!(
                    "default route handled successfully for connection {}",
                    connection_id
                );
            } else {
                error!(
                    "default route failed for connection {}: {:?}",
                    connection_id, res
                );
            }
            res
        }
    };

    match result {
        Ok(_) => {
            info!(
                "Handler completed successfully for route {} and connection {}",
                route, connection_id
            );
            Ok(ApiGatewayProxyResponse {
                status_code: 200,
                headers: Default::default(),
                multi_value_headers: Default::default(),
                body: None,
                is_base64_encoded: false,
            })
        }
        Err(e) => {
            error!(
                "Handler failed for route {} and connection {}: {:?}",
                route, connection_id, e
            );
            Ok(ApiGatewayProxyResponse {
                status_code: 500,
                headers: Default::default(),
                multi_value_headers: Default::default(),
                body: Some(format!("{{\"message\": \"Internal server error\"}}").into()),
                is_base64_encoded: false,
            })
        }
    }
}
