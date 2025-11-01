use aws_lambda_events::apigw::ApiGatewayWebsocketProxyRequestContext;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct Jwk {
    pub kid: String,
    pub kty: String,
    pub n: String,
    pub e: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Jwks {
    pub keys: Vec<Jwk>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthPolicy {
    #[serde(rename = "principalId")]
    pub principal_id: String,
    #[serde(rename = "policyDocument")]
    pub policy_document: PolicyDocument,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PolicyDocument {
    #[serde(rename = "Version")]
    pub version: String,
    #[serde(rename = "Statement")]
    pub statement: Vec<IamPolicyStatement>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IamPolicyStatement {
    #[serde(rename = "Action")]
    pub action: Vec<String>,
    #[serde(rename = "Effect")]
    pub effect: String,
    #[serde(rename = "Resource")]
    pub resource: Vec<String>,
}

impl AuthPolicy {
    pub fn allow(principal_id: String, resource: String) -> Self {
        // Extract the base ARN and replace the route with a wildcard
        // From: arn:aws:execute-api:region:account:api-id/stage/$connect
        // To:   arn:aws:execute-api:region:account:api-id/stage/*
        let wildcard_resource = if let Some(idx) = resource.rfind('/') {
            format!("{}/*", &resource[..idx])
        } else {
            resource.clone()
        };

        AuthPolicy {
            principal_id: principal_id.clone(),
            policy_document: PolicyDocument {
                version: "2012-10-17".to_string(),
                statement: vec![IamPolicyStatement {
                    action: vec!["execute-api:Invoke".to_string()],
                    effect: "Allow".to_string(),
                    resource: vec![wildcard_resource],
                }],
            },
            context: Some(serde_json::json!({
                "userId": principal_id
            })),
        }
    }

    pub fn deny() -> Self {
        AuthPolicy {
            principal_id: "user".to_string(),
            policy_document: PolicyDocument {
                version: "2012-10-17".to_string(),
                statement: vec![IamPolicyStatement {
                    action: vec!["execute-api:Invoke".to_string()],
                    effect: "Deny".to_string(),
                    resource: vec!["*".to_string()],
                }],
            },
            context: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WebsocketAuthorizerEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    #[serde(rename = "methodArn")]
    pub method_arn: String,
    pub headers: Option<HashMap<String, String>>,
    #[serde(rename = "multiValueHeaders")]
    pub multi_value_headers: Option<HashMap<String, Vec<String>>>,
    #[serde(rename = "queryStringParameters")]
    pub query_string_parameters: Option<HashMap<String, String>>,
    #[serde(rename = "multiValueQueryStringParameters")]
    pub multi_value_query_string_parameters: Option<HashMap<String, Vec<String>>>,
    #[serde(rename = "requestContext")]
    pub request_context: Option<ApiGatewayWebsocketProxyRequestContext>,
    #[serde(rename = "stageVariables")]
    pub stage_variables: Option<HashMap<String, String>>,
}
