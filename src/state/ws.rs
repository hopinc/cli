use crate::config::HOP_LEAP_EDGE_URL;
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tokio_tungstenite::{
    connect_async, tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream,
};

#[derive(Debug, Deserialize, Serialize, Clone)]
enum OpCodes {
    Dispatch = 0,
    Hello,
    Identify,
    Heartbeat,
    HeartbeatAck,
}

#[derive(Debug, Deserialize, Serialize, Clone)]

struct SocketMessage<T> {
    op: OpCodes,
    d: Option<T>,
}

#[derive(Debug, Deserialize, Clone)]
struct LEAuthParams {
    project_id: String,
    token: String,
}

pub struct WebsocketClient {
    pub token: String,
    pub heartbeat_interval: Option<tokio::time::Interval>,
    pub last_heartbeat_acknowledged: bool,
    pub sender: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    pub reciever: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
}

impl WebsocketClient {
    const HEARTBEAT: SocketMessage<()> = SocketMessage {
        op: OpCodes::Heartbeat,
        d: None,
    };

    pub async fn new(token: &str) -> Self {
        let last_heartbeat_acknowledged = true;

        let (socket, _) = connect_async(HOP_LEAP_EDGE_URL)
            .await
            .expect("Error connecting to Hop Leap Edge");

        // TODO: run these in threads and etc
        let (sender, reciever) = socket.split();

        Self {
            token: token.to_string(),
            heartbeat_interval: None,
            last_heartbeat_acknowledged,
            sender,
            reciever,
        }
    }

    async fn send_heartbeat(&mut self) {}

    async fn idenitfy(&mut self) {}
}
