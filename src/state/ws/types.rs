use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

pub type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

#[derive(Debug, Serialize_repr, Deserialize_repr, Clone)]
#[repr(u8)]
pub enum OpCodes {
    Dispatch = 0,
    Hello = 1,
    Identify = 2,
    Heartbeat = 3,
    HeartbeatAck = 4,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SocketMessage<T> {
    pub op: OpCodes,
    pub d: Option<T>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SocketHello {
    pub heartbeat_interval: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LeapEdgeAuthParams {
    pub project_id: String,
    pub token: String,
}
