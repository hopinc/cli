use crate::types::Secret;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Clone)]
pub struct SecretResponse {
    pub secret: Secret,
}

#[derive(Debug, Serialize)]
pub struct CreateParams {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Serialize)]
pub struct UpdateParams {
    pub value: String,
}
