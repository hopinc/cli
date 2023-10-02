use async_tungstenite::tokio::ConnectStream;
use async_tungstenite::WebSocketStream;
use chrono::DateTime;
use serde::de::Error as SerdeError;
use serde::Deserialize;
use serde_json::Value;
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::commands::containers::types::{Log, Metrics};

pub type WsStream = WebSocketStream<ConnectStream>;

#[derive(Debug, Clone, Copy, Deserialize_repr, Serialize_repr)]
#[repr(u8)]
pub enum OpCode {
    Hello = 1,
    Identify,
    ServiceMessage,
    Heartbeat,
    Logs,
    Stdin, // unused for now
    HeartbeatAck,
    RequestMetrics,
    RequestMetricsAck,
    Metrics,
    RequestLogs,
    RequestLogsAck,
    UnsubscribeLogs,
    UnsubscribeMetrics,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ConnectionStage {
    Connected,
    Handshake,
    Identifying,
    Disconnected,
}

#[derive(Debug, Clone)]
pub enum ArisuEvent {
    Hello(u64),
    ServiceMessage(String),
    Logs(Log),
    Metrics(Metrics),
    HeartbeatAck,
    LogsRequestedAck,
    MetricsRequestedAck,
}

impl<'de> Deserialize<'de> for ArisuEvent {
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

                Self::Hello(heartbeat_interval)
            }

            OpCode::ServiceMessage => {
                let d = gw_event
                    .remove("d")
                    .ok_or_else(|| SerdeError::custom("missing d"))?;

                let message = d
                    .get("message")
                    .and_then(Value::as_str)
                    .ok_or_else(|| SerdeError::custom("missing message"))?
                    .to_string();

                Self::ServiceMessage(message)
            }

            OpCode::Logs => {
                let d = gw_event
                    .remove("d")
                    .ok_or_else(|| SerdeError::custom("missing d"))?;

                let timestamp = d
                    .get("timestamp")
                    .and_then(Value::as_str)
                    .ok_or_else(|| SerdeError::custom("missing timestamp"))?
                    .to_string();

                let timestamp = DateTime::parse_from_rfc3339(&timestamp)
                    .map_err(|_| SerdeError::custom("invalid timestamp"))?
                    .with_timezone(&chrono::Utc);

                let message = d
                    .get("data")
                    .and_then(Value::as_str)
                    .ok_or_else(|| SerdeError::custom("missing data"))?
                    .to_string();

                let level = d
                    .get("level")
                    .and_then(Value::as_str)
                    .ok_or_else(|| SerdeError::custom("missing level"))?
                    .to_string();

                Self::Logs(Log {
                    timestamp,
                    message,
                    level,
                })
            }

            OpCode::Metrics => {
                let d = gw_event
                    .remove("d")
                    .ok_or_else(|| SerdeError::custom("missing d"))?;

                let cpu_usage_percent = d
                    .get("cpu_usage_percent")
                    .and_then(Value::as_f64)
                    .ok_or_else(|| SerdeError::custom("missing cpu_usage_percent"))?;

                let memory_usage_bytes = d
                    .get("memory_usage_bytes")
                    .and_then(Value::as_u64)
                    .ok_or_else(|| SerdeError::custom("missing memory_usage_bytes"))?;

                Self::Metrics(Metrics {
                    cpu_usage_percent,
                    memory_usage_bytes,
                })
            }

            OpCode::HeartbeatAck => Self::HeartbeatAck,

            OpCode::RequestLogsAck => Self::LogsRequestedAck,

            OpCode::RequestMetricsAck => Self::MetricsRequestedAck,

            _ => return Err(SerdeError::custom("invalid opcode")),
        };

        Ok(event)
    }
}

#[derive(Debug, Clone)]
pub enum ArisuMessage {
    Open,
    Logs(Log),
    Metrics(Metrics),

    ServiceMessage(String),
}

impl From<Log> for ArisuMessage {
    fn from(log: Log) -> Self {
        Self::Logs(log)
    }
}

impl From<Metrics> for ArisuMessage {
    fn from(metrics: Metrics) -> Self {
        Self::Metrics(metrics)
    }
}
