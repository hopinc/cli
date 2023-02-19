use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum KeyType {
    Key,
    Totp,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum LoginResponse {
    Success {
        token: String,
    },
    SecondFactorRequired {
        ticket: String,
        preferred_type: KeyType,
        types: Vec<KeyType>,
    },
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum SecondFactorRequest {
    Totp { code: String, ticket: String },
}
