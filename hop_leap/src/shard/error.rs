use std::error::Error as StdError;
use std::fmt;

use async_tungstenite::tungstenite::protocol::CloseFrame;

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum Error {
    Closed(Option<CloseFrame<'static>>),
    ExpectedHello,
    HeartbeatFailed,
    InvalidAuthentication,
    InvalidHandshake,
    InvalidOpCode,
    ReconnectFailure,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Closed(_) => f.write_str("Connection closed"),
            Self::ExpectedHello => f.write_str("Expected a Hello"),
            Self::HeartbeatFailed => f.write_str("Failed sending a heartbeat"),
            Self::InvalidAuthentication => f.write_str("Sent invalid authentication"),
            Self::InvalidHandshake => f.write_str("Expected a valid Handshake"),
            Self::InvalidOpCode => f.write_str("Invalid OpCode"),
            Self::ReconnectFailure => f.write_str("Failed to Reconnect"),
        }
    }
}

impl StdError for Error {}
