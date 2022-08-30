use std::fmt;

use serde::de::Error as SerdeError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_repr::Deserialize_repr;

use crate::leap::types::Event;

#[derive(Clone, Debug)]
pub enum InterMessage {
    Json(Value),
    Close,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ConnectionStage {
    Connected,
    Handshake,
    Identifying,
    Disconnected,
}

impl ConnectionStage {
    pub fn is_connecting(&self) -> bool {
        matches!(self, Self::Handshake | Self::Identifying)
    }

    pub fn is_connected(&self) -> bool {
        matches!(self, Self::Connected)
    }
}

impl fmt::Display for ConnectionStage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match *self {
            Self::Connected => "connected",
            Self::Handshake => "handshaking",
            Self::Identifying => "identifying",
            Self::Disconnected => "disconnected",
        })
    }
}

#[derive(Debug)]
pub enum ShardAction {
    Heartbeat(Option<String>),
    Identify,
    Reconnect(ReconnectType),
    Update,
}

#[derive(Debug)]
pub enum ReconnectType {
    /// send IDENTIFY.
    Reidentify,
}

#[derive(Debug, Clone, Copy, Deserialize_repr)]
#[repr(u8)]
pub enum OpCode {
    Dispatch = 0,
    Hello = 1,
    Identify = 2,
    Heartbeat = 3,
    HeartbeatAck = 4,
    Unknown = !0,
}

impl OpCode {
    pub fn number(self) -> u8 {
        self as u8
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum GatewayEvent {
    /// server hello
    Hello(u64),
    /// general dispatch event
    Dispatch(Event),
    /// heartbeats with optional tag
    Heartbeat(Option<String>),
    /// heartbeat ack with optional tag and latency
    HeartbeatAck(Option<String>, Option<u64>),
}

impl<'de> Deserialize<'de> for GatewayEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut gw_event = serde_json::Map::deserialize(deserializer)?;

        let op_code = gw_event
            .remove("op")
            .ok_or_else(|| SerdeError::custom("missing op code"))
            .and_then(OpCode::deserialize)
            .map_err(SerdeError::custom)?;

        let event = match op_code {
            OpCode::Hello => {
                let d = gw_event
                    .remove("d")
                    .ok_or_else(|| SerdeError::custom("missing d"))?;

                let heartbeat_interval = d
                    .get("heartbeat_interval")
                    .and_then(serde_json::Value::as_u64)
                    .ok_or_else(|| SerdeError::custom("missing heartbeat_interval"))?;

                GatewayEvent::Hello(heartbeat_interval)
            }

            OpCode::Dispatch => {
                let event = gw_event
                    .remove("d")
                    .ok_or_else(|| SerdeError::custom("missing event data"))
                    .and_then(Event::deserialize)
                    .map_err(SerdeError::custom)?;

                GatewayEvent::Dispatch(event)
            }

            OpCode::Heartbeat => {
                // heartbeats sent by the gateway can have a tag
                let d = gw_event
                    .remove("d")
                    .map(serde_json::Map::deserialize)
                    .transpose()
                    .map_err(SerdeError::custom)?;

                let tag = d
                    .and_then(|mut d| d.remove("tag"))
                    .map(|tag| tag.as_str().unwrap().to_string());

                GatewayEvent::Heartbeat(tag)
            }

            OpCode::HeartbeatAck => {
                // heartbeat acks sent by the gateway can have a tag and latency
                let d = gw_event
                    .remove("d")
                    .map(serde_json::Map::deserialize)
                    .transpose()
                    .map_err(SerdeError::custom)?;

                let tag = d
                    .clone()
                    .and_then(|mut d| d.remove("tag"))
                    .map(|tag| tag.as_str().unwrap().to_string());

                let latency = d
                    .and_then(|mut d| d.remove("latency"))
                    .and_then(|latency| latency.as_u64());
                GatewayEvent::HeartbeatAck(tag, latency)
            }

            _ => return Err(SerdeError::custom("invalid opcode")),
        };

        Ok(event)
    }
}
