use std::{io::Cursor, time::Duration};

use crate::config::{HOP_LEAP_EDGE_PROJECT_ID, HOP_LEAP_EDGE_URL};
use async_compression::tokio::bufread::ZlibDecoder;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_repr::{Deserialize_repr, Serialize_repr};
use tokio::{io::AsyncReadExt, spawn, sync::mpsc, task::JoinHandle, time::interval};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

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
    recv: mpsc::Receiver<Value>,
}

#[derive(Debug)]
pub struct WebsocketClient {
    pub auth: Option<LEAuthParams>,
    pub last_heartbeat_acknowledged: bool,
    pub thread: Option<JoinHandle<()>>,
    channels: Option<SocketChannels>,
}

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

    pub async fn connect(mut self, token: &str) -> Self {
        let (sender_outbound, mut reciever_outbound) = mpsc::channel::<String>(1);
        let (sender_inbound, receiver_inbound) = mpsc::channel::<Value>(1);

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
            let (socket, _) = connect_async(HOP_LEAP_EDGE_URL)
                .await
                .expect("Error connecting to Hop Leap Edge");

            let (mut sender, mut reciever) = socket.split();

            // the first message has to be server hello so lets wait for it
            let hello = reciever
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
                    // gateway reciever
                    message = reciever.next() => {
                        match message {
                            Some(recieved) => match recieved {
                                Ok(message) => match Self::parse_message::<SocketMessage<Value>>(message).await {
                                    SocketMessage { op: OpCodes::HeartbeatAck, d: _ } => {
                                        self.last_heartbeat_acknowledged = true;
                                    }

                                    SocketMessage { op: OpCodes::Dispatch, d: data } => {
                                        sender_inbound.send(data.unwrap()).await.unwrap();
                                    }

                                    // ignore other messages
                                    _ => {}

                                },
                                Err(_) => {
                                    panic!("type chec")
                                }
                            },
                            None => {
                                println!("type check");
                            }
                        }
                    },

                    // internal rcv thread
                    internal = reciever_outbound.recv() => {
                        match internal {
                            Some(message) => match message.as_str() {
                                "CLOSE" => {
                                    sender.close().await.unwrap();
                                },

                                message => sender.send(message.into()).await.expect("Error sending message")
                            },
                            None => panic!("type check")
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

        // set thread to self to close it later lol!
        self.thread = Some(thread);

        self
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
        let value = self.channels.as_mut().unwrap().recv.recv().await;
        value.map(|v| serde_json::from_value(v).expect("Failed to parse message"))
    }

    pub async fn _send_message<T>(&mut self, message: T)
    where
        T: serde::ser::Serialize + std::fmt::Debug,
    {
        let message = serde_json::to_string(&message).unwrap();
        self.channels
            .as_mut()
            .expect("not connected bruh")
            .send
            .send(message)
            .await
            .unwrap();
    }

    pub async fn close(&mut self) {
        self.channels
            .as_mut()
            .expect("not connected bruh")
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
