use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub token_type: String,
    pub expires_in: i64,
}

#[derive(Deserialize, Serialize)]
pub struct TokenClaims {
    pub sub: String, // subject (user ID)
    pub exp: usize,  // expiration time
    pub iat: usize,  // issued at
}
