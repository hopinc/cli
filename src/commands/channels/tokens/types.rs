use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct LeapToken {
    pub id: String,
    pub created_at: String,
    pub state: Option<Value>,
    pub expires_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateLeapToken {
    pub expires_at: Option<String>,
    pub state: Option<Value>,
}

#[derive(Debug, Deserialize)]
pub struct SingleLeapToken {
    pub token: LeapToken,
}

#[derive(Debug, Deserialize)]
pub struct MultipleLeapToken {
    pub tokens: Vec<LeapToken>,
}
