use std::io::Cursor;
use std::time::Duration;

use async_compression::tokio::bufread::ZlibDecoder;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_repr::{Deserialize_repr, Serialize_repr};
use tokio::io::AsyncReadExt;
use tokio::spawn;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio::time::interval;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message;

use crate::config::{HOP_LEAP_EDGE_PROJECT_ID, HOP_LEAP_EDGE_URL};

#[derive(Debug, Serialize_repr, Deserialize_repr, Clone)]
#[repr(u8)]
enum OpCodes {
    Dispatch = 0,
    Hello = 1,
    Identify = 2,
    Heartbeat = 3,
    HeartbeatAck = 4,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct SocketMessage<T> {
    op: OpCodes,
    d: Option<T>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct SocketHello {
    heartbeat_interval: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LEAuthParams {
    project_id: String,
    token: String,
}

#[derive(Debug)]
pub struct SocketChannels {
    send: mpsc::Sender<String>,
    recv: mpsc::Receiver<String>,
}

#[derive(Debug)]
pub struct WebsocketClient {
    pub auth: Option<LEAuthParams>,
    pub last_heartbeat_acknowledged: bool,
    pub thread: Option<JoinHandle<()>>,
    channels: Option<SocketChannels>,
}

#[derive(Debug, Clone)]
pub struct WebsocketError(String);

impl WebsocketClient {
    pub fn new() -> Self {
        let last_heartbeat_acknowledged = true;

        Self {
            auth: None,
            thread: None,
            channels: None,
            last_heartbeat_acknowledged,
        }
    }

    pub async fn connect(mut self, token: &str) -> Result<Self, WebsocketError> {
        let (sender_outbound, mut receiver_outbound) = mpsc::channel::<String>(1);
        let (sender_inbound, receiver_inbound) = mpsc::channel::<String>(1);

        // prepare client for sending / receiving messages
        self.auth = Some(LEAuthParams {
            project_id: HOP_LEAP_EDGE_PROJECT_ID.to_string(),
            token: token.to_string(),
        });
        self.channels = Some(SocketChannels {
            send: sender_outbound,
            recv: receiver_inbound,
        });

        let socket_auth = self.auth.clone();

        // start massive thread to get messages / deliver messages
        let thread = spawn(async move {
            let (socket, _) = connect_async(HOP_LEAP_EDGE_URL).await.unwrap();

            let (mut sender, mut receiver) = socket.split();

            // the first message has to be server hello so lets wait for it
            let hello = receiver
                .next()
                .await
                .expect("Error reading from socket")
                .expect("Error reading from socket");

            let hello: SocketMessage<SocketHello> = Self::parse_message(hello).await;

            // it is safe to unwrap since first message **has** to be hello
            let htb = hello.d.unwrap().heartbeat_interval;

            let mut interval = interval(Duration::from_millis(htb));

            // skip first htb
            interval.tick().await;

            sender
                .send(
                    serde_json::to_string(&SocketMessage {
                        op: OpCodes::Identify,
                        d: Some(socket_auth),
                    })
                    .unwrap()
                    .into(),
                )
                .await
                .unwrap();

            loop {
                tokio::select! {
                    // gateway receiver
                    message = receiver.next() => {
                        match message {
                            Some(recieved) => match recieved {
                                Ok(message) => match Self::parse_message::<SocketMessage<Value>>(message).await {
                                    SocketMessage { op: OpCodes::HeartbeatAck, d: _ } => {
                                        self.last_heartbeat_acknowledged = true;
                                    }

                                    SocketMessage { op: OpCodes::Dispatch, d: data } => {
                                        match sender_inbound.send(serde_json::to_string(&data).unwrap()).await {
                                            Ok(_) => {}
                                            // channel was closed before the message was delievered
                                            // no need to panic here
                                            Err(_) => {}
                                        }
                                    }

                                    // ignore other messages
                                    _ => {}
                                },
                                // message recieved was not valid json / packed incorrectly
                                Err(err) => {
                                    panic!("Connection error, {}", err)
                                }
                            },

                            // no idea why this would happen
                            None => {}
                        }
                    },

                    // internal rcv thread
                    internal = receiver_outbound.recv() => {
                        match internal {
                            Some(message) => match message.as_str() {
                                "CLOSE" => {
                                    sender.close().await.unwrap();
                                },

                                message => sender.send(message.into()).await.expect("Error sending message")
                            },
                            // no idea why this would happen
                            None => {}
                        }
                    },

                    // heartbeat sender
                    _ = interval.tick() => {
                        // println!("DEBUG: Sending heartbeat");
                        let heartbeat: SocketMessage<()> = SocketMessage {
                            op: OpCodes::Heartbeat,
                            d: None,
                        };

                        sender.send(serde_json::to_string(&heartbeat).unwrap().into()).await.expect("Error sending heartbeat");
                    }
                }
            }
        });

        self.thread = Some(thread);

        Ok(self)
    }

    async fn parse_message<T>(message: Message) -> T
    where
        T: serde::de::DeserializeOwned + std::fmt::Debug,
    {
        match message {
            Message::Text(text) => serde_json::from_str(&text).expect("Failed to parse message"),
            Message::Binary(bin) => {
                let mut uncompressed = ZlibDecoder::new(Cursor::new(bin));
                let mut buff = vec![];
                uncompressed
                    .read_to_end(&mut buff)
                    .await
                    .expect("Failed to read message");

                serde_json::from_slice(&buff).expect("Failed to deflate message")
            }
            _ => {
                panic!("received unexpected message type");
            }
        }
    }

    pub async fn recieve_message<T>(&mut self) -> Option<T>
    where
        T: serde::de::DeserializeOwned,
    {
        match self.channels {
            Some(ref mut channels) => match channels.recv.recv().await {
                Some(message) => match message.as_str() {
                    message => Some(serde_json::from_str(message).unwrap()),
                },
                None => None,
            },
            None => None,
        }
    }

    pub async fn _send_message<T>(&mut self, message: T)
    where
        T: serde::ser::Serialize + std::fmt::Debug,
    {
        if self.channels.is_none() {
            panic!("Client not connected");
        }

        let message = serde_json::to_string(&message).unwrap();
        self.channels
            .as_mut()
            .unwrap()
            .send
            .send(message)
            .await
            .unwrap();
    }

    pub async fn close(&mut self) {
        if self.channels.is_none() {
            // do nothing;
            return ();
        }

        self.channels
            .as_mut()
            .unwrap()
            .send
            .send("CLOSE".to_string())
            .await
            .unwrap();

        self.thread.as_ref().unwrap().abort();

        self.thread = None;
        self.channels = None;
        self.auth = None;
        self.last_heartbeat_acknowledged = false;
    }
}
