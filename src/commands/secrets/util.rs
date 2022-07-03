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

pub fn validate_name(name: &str) -> Result<(), String> {
    let regex = regex::Regex::new(r"^[a-zA-Z0-9_]{1,64}$").unwrap();

    if regex.is_match(name) {
        Ok(())
    } else {
        panic!("Invalid name. Secret names are limited to 64 characters in length, must be alphanumeric (with underscores) and are automatically uppercased.");
    }
}
