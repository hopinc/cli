use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "e")]
pub enum Event {
    #[serde(rename = "INIT")]
    Init(EventCapsule<InitEvent>),
    #[serde(rename = "MESSAGE")]
    Message(EventCapsule<Value>),
    #[serde(rename = "DIRECT_MESSAGE")]
    DirectMessage(EventCapsule<Value>),
    #[serde(rename = "SUBSCRIBE")]
    Subscribe(EventCapsule<Value>),
    #[serde(rename = "AVAILABLE")]
    Available(EventCapsule<Channel>),
    #[serde(rename = "UNAVAILABLE")]
    Unavailable(EventCapsule<Value>),
    #[serde(rename = "PIPE_ROOM_AVAILABLE")]
    PipeRoomAvailable(EventCapsule<Value>),
    #[serde(rename = "PIPE_ROOM_UPDATE")]
    PipeRoomUpdate(EventCapsule<Value>),
    #[serde(rename = "STATE_UPDATE")]
    StateUpdate(EventCapsule<Value>),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EventCapsule<T> {
    #[serde(rename = "c")]
    pub channel: Option<String>,
    #[serde(rename = "d")]
    pub data: T,
    #[serde(rename = "u", skip_serializing)]
    pub unicast: bool,
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
    pub channels: Vec<Channel>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Channel {
    pub capabilities: Option<Value>,
    pub id: String,
    pub project_id: String,
    pub state: Value,
    #[serde(rename = "type")]
    pub type_: ChannelType,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ChannelType {
    #[serde(rename = "private")]
    Private,
    #[serde(rename = "public")]
    Public,
    #[serde(rename = "unprotected")]
    Unprotected,
}
