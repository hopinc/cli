use std::fmt;

use serde::Deserialize;
use serde_json::Value;

#[derive(Clone, Debug)]
pub enum InterMessage {
    #[cfg(feature = "client")]
    Client(Box<ShardClientMessage>),
    Json(Value),
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum ConnectionStage {
    Connected,
    Connecting,
    Disconnected,
    Handshake,
    Identifying,
    Resuming,
}

impl fmt::Display for ConnectionStage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match *self {
            Self::Connected => "connected",
            Self::Connecting => "connecting",
            Self::Disconnected => "disconnected",
            Self::Handshake => "handshaking",
            Self::Identifying => "identifying",
            Self::Resuming => "resuming",
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
#[repr(u8)]
pub enum OpCode {
    Dispatch = 0,
    Hello = 1,
    Identify = 2,
    Heartbeat = 3,
    HeartbeatAck = 4,
}
