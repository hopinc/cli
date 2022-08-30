use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "e")]
pub enum Event {
    #[serde(rename = "INIT")]
    Init(Value),
    #[serde(rename = "MESSAGE")]
    Message(Value),
    #[serde(rename = "DIRECT_MESSAGE")]
    DirectMessage(Value),
    #[serde(rename = "SUBSCRIBE")]
    Subscribe(Value),
    #[serde(rename = "AVAILABLE")]
    Available(Value),
    #[serde(rename = "UNAVAILABLE")]
    Unavailable(Value),
    #[serde(rename = "PIPE_ROOM_AVAILABLE")]
    PipeRoomAvailable(Value),
    #[serde(rename = "PIPE_ROOM_UPDATE")]
    PipeRoomUpdate(Value),
    #[serde(rename = "STATE_UPDATE")]
    StateUpdate(Value),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ConnectionScopes {
    #[serde(rename = "project")]
    Project,
    #[serde(rename = "token")]
    Token,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InitEvent {
    pub cid: String,
    pub connection_count: u64,
    pub metadata: Option<Value>,
    pub scope: ConnectionScopes,
    pub channels: Vec<Value>,
}
