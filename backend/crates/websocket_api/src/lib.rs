pub mod connections;
pub mod handlers;
pub mod models;
pub mod queue;

use aws_sdk_dynamodb::Client as DynamoClient;
use tracing::info;

#[derive(Clone)]
pub struct AppState {
    pub dynamodb: DynamoClient,
    pub queue_table: String,
    pub connections_table: String,
    pub region: String,
    pub websocket_api_endpoint: String,
}

impl AppState {
    pub async fn new() -> Self {
        let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
        let dynamodb = DynamoClient::new(&config);
        let queue_table = std::env::var("QUEUE_TABLE").expect("QUEUE_TABLE must be set");
        let connections_table =
            std::env::var("CONNECTIONS_TABLE").expect("CONNECTIONS_TABLE must be set");
        let region = std::env::var("AWS_REGION").unwrap_or("eu-west-1".to_string());
        let websocket_api_endpoint =
            std::env::var("WEBSOCKET_API_ENDPOINT").expect("WEBSOCKET_API_ENDPOINT must be set");
        info!(
            "Initialized AppState with queue_table={}, connections_table={}, region={}, websocket_api_endpoint={}",
            queue_table, connections_table, region, websocket_api_endpoint
        );
        Self {
            dynamodb,
            queue_table,
            connections_table,
            region,
            websocket_api_endpoint,
        }
    }
}
