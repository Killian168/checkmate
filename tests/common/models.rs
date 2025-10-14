use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct CreateUserPayload<'a> {
    pub email: &'a str,
    pub password: &'a str,
    pub first_name: &'a str,
    pub last_name: &'a str,
}

#[derive(Serialize)]
pub struct LoginPayload<'a> {
    pub email: &'a str,
    pub password: &'a str,
}

#[derive(Deserialize)]
pub struct LoginResponse {
    pub token: String,
}

#[derive(Deserialize)]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub created_at: String,
}
