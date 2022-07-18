use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct Data {
    pub d: Option<String>,
    pub e: String,
}

#[derive(Debug, Deserialize)]
pub struct Message {
    pub d: Value,
    pub e: String,
}
