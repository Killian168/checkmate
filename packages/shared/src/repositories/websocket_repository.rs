use async_trait::async_trait;
use aws_sdk_apigatewaymanagement::{primitives::Blob, Client as ApiGatewayClient};
use aws_sdk_dynamodb::Client as DynamoDbClient;
use std::env;
use tracing::info;

#[async_trait]
pub trait WebSocketRepository: Send + Sync {
    async fn store_connection(
        &self,
        player_id: &str,
        connection_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    async fn remove_connection(
        &self,
        player_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    async fn remove_connection_by_id(
        &self,
        connection_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    async fn get_connection_id(
        &self,
        player_id: &str,
    ) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>>;

    async fn send_message(
        &self,
        connection_id: &str,
        message: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

pub struct DynamoDbWebSocketRepository {
    dynamodb_client: DynamoDbClient,
    api_gateway_client: ApiGatewayClient,
    table_name: String,
}

impl DynamoDbWebSocketRepository {
    pub fn new(dynamodb_client: DynamoDbClient, api_gateway_client: ApiGatewayClient) -> Self {
        let table_name = env::var("PLAYER_CONNECTIONS_TABLE")
            .expect("PLAYER_CONNECTIONS_TABLE environment variable must be set");

        Self {
            dynamodb_client,
            api_gateway_client,
            table_name,
        }
    }

    fn get_api_gateway_endpoint(&self) -> String {
        // For WebSocket API Gateway, we need to construct the endpoint URL
        // Format: https://{api-id}.execute-api.{region}.amazonaws.com/{stage}
        // We'll get this from environment variables or use a default
        if let Ok(endpoint) = env::var("WEBSOCKET_API_ENDPOINT") {
            endpoint
        } else {
            // Fallback: construct from other environment variables
            let region = env::var("AWS_REGION").unwrap_or_else(|_| "eu-west-1".to_string());
            let api_id = env::var("WEBSOCKET_API_ID").unwrap_or_else(|_| "yphq15v1gk".to_string());
            let stage = env::var("STAGE").unwrap_or_else(|_| "dev".to_string());

            format!(
                "https://{}.execute-api.{}.amazonaws.com/{}",
                api_id, region, stage
            )
        }
    }
}

#[async_trait]
impl WebSocketRepository for DynamoDbWebSocketRepository {
    async fn store_connection(
        &self,
        player_id: &str,
        connection_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.dynamodb_client
            .put_item()
            .table_name(&self.table_name)
            .item(
                "player_id",
                aws_sdk_dynamodb::types::AttributeValue::S(player_id.to_string()),
            )
            .item(
                "connection_id",
                aws_sdk_dynamodb::types::AttributeValue::S(connection_id.to_string()),
            )
            .send()
            .await?;

        info!("Stored WebSocket connection for player: {}", player_id);
        Ok(())
    }

    async fn remove_connection(
        &self,
        player_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.dynamodb_client
            .delete_item()
            .table_name(&self.table_name)
            .key(
                "player_id",
                aws_sdk_dynamodb::types::AttributeValue::S(player_id.to_string()),
            )
            .send()
            .await?;

        info!("Removed WebSocket connection for player: {}", player_id);
        Ok(())
    }

    async fn remove_connection_by_id(
        &self,
        connection_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Scan for the player_id associated with this connection_id
        let scan_result = self
            .dynamodb_client
            .scan()
            .table_name(&self.table_name)
            .filter_expression("connection_id = :connection_id")
            .expression_attribute_values(
                ":connection_id",
                aws_sdk_dynamodb::types::AttributeValue::S(connection_id.to_string()),
            )
            .send()
            .await?;

        if let Some(items) = scan_result.items {
            for item in items {
                if let Some(player_id_attr) = item.get("player_id") {
                    if let aws_sdk_dynamodb::types::AttributeValue::S(player_id) = player_id_attr {
                        info!("Removing connection for player: {}", player_id);

                        self.dynamodb_client
                            .delete_item()
                            .table_name(&self.table_name)
                            .key(
                                "player_id",
                                aws_sdk_dynamodb::types::AttributeValue::S(player_id.clone()),
                            )
                            .send()
                            .await?;
                    }
                }
            }
        }

        info!("Removed WebSocket connection by ID: {}", connection_id);
        Ok(())
    }

    async fn get_connection_id(
        &self,
        player_id: &str,
    ) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        let result = self
            .dynamodb_client
            .get_item()
            .table_name(&self.table_name)
            .key(
                "player_id",
                aws_sdk_dynamodb::types::AttributeValue::S(player_id.to_string()),
            )
            .send()
            .await?;

        if let Some(item) = result.item {
            if let Some(connection_id_attr) = item.get("connection_id") {
                if let aws_sdk_dynamodb::types::AttributeValue::S(connection_id) =
                    connection_id_attr
                {
                    return Ok(Some(connection_id.clone()));
                }
            }
        }

        Ok(None)
    }

    async fn send_message(
        &self,
        connection_id: &str,
        message: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Create a new API Gateway client with the correct endpoint
        let config = aws_config::load_from_env().await;
        let api_gateway_config = aws_sdk_apigatewaymanagement::config::Builder::from(&config)
            .endpoint_url(self.get_api_gateway_endpoint())
            .build();
        let api_gateway_client =
            aws_sdk_apigatewaymanagement::Client::from_conf(api_gateway_config);

        api_gateway_client
            .post_to_connection()
            .connection_id(connection_id)
            .data(Blob::new(message.as_bytes()))
            .send()
            .await?;

        info!("Sent message to connection: {}", connection_id);
        Ok(())
    }
}
