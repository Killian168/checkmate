use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct JoinQueueRequest {
    pub queue_type: String,
}
