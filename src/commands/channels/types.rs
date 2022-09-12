use std::{fmt::Display, str::FromStr};

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum ChannelType {
    #[serde(rename = "public")]
    Public,
    #[serde(rename = "private")]
    Private,
    #[serde(rename = "unprotected")]
    Unprotected,
}

impl FromStr for ChannelType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        serde_json::from_str(&format!("\"{}\"", s.to_lowercase())).map_err(|e| anyhow!(e))
    }
}

impl Display for ChannelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self).unwrap().replace('"', "")
        )
    }
}

impl ChannelType {
    pub fn variants() -> Vec<Self> {
        vec![Self::Public, Self::Private, Self::Unprotected]
    }
}

#[derive(Debug, Deserialize)]
pub struct Channel {
    pub id: String,
    #[serde(rename = "type")]
    pub type_: ChannelType,
    pub created_at: String,
    // state is user specified
    pub state: Value,
}

#[derive(Debug, Serialize)]
pub struct CreateChannel {
    #[serde(rename = "type")]
    pub type_: ChannelType,
    pub state: Value,
}

#[derive(Debug, Deserialize)]
pub struct SingleChannel {
    pub channel: Channel,
}

#[derive(Debug, Deserialize)]
pub struct MultipleChannels {
    pub channels: Vec<Channel>,
}
