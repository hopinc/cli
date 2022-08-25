use std::error::Error as StdError;
use std::fmt;
use std::io::Error as IoError;

use async_tungstenite::tungstenite::error::Error as TungsteniteError;
use serde_json::Error as JsonError;

use crate::shard::error::Error as LeapError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Io(IoError),
    Json(JsonError),
    Leap(LeapError),
    Tungstenite(TungsteniteError),
}

impl From<IoError> for Error {
    fn from(e: IoError) -> Self {
        Self::Io(e)
    }
}

impl From<JsonError> for Error {
    fn from(e: JsonError) -> Self {
        Self::Json(e)
    }
}

impl From<LeapError> for Error {
    fn from(e: LeapError) -> Self {
        Self::Leap(e)
    }
}

impl From<TungsteniteError> for Error {
    fn from(e: TungsteniteError) -> Self {
        Self::Tungstenite(e)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(inner) => fmt::Display::fmt(&inner, f),
            Self::Json(inner) => fmt::Display::fmt(&inner, f),
            Self::Tungstenite(inner) => fmt::Display::fmt(&inner, f),
            Self::Leap(inner) => fmt::Display::fmt(&inner, f),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Io(inner) => Some(inner),
            Self::Json(inner) => Some(inner),
            Self::Tungstenite(inner) => Some(inner),
            Self::Leap(inner) => Some(inner),
        }
    }
}
