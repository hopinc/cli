
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Base<T> {
    pub success: bool,
    pub data: T,
}

#[derive(Debug, Deserialize)]
pub struct ErrorContent {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct ErrorResponse {
    pub error: ErrorContent,
}
