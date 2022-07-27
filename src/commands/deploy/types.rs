use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct EventData {
    pub progress: Option<String>,
    pub deployment_id: String,
}

#[derive(Debug, Deserialize)]
pub struct Event {
    pub d: Option<EventData>,
    pub e: String,
}

#[derive(Debug, Deserialize)]
pub struct Message {
    pub d: Value,
    pub e: String,
}
