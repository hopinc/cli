use std::fmt::Display;
use std::str::FromStr;

use anyhow::{anyhow, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::utils::validate_json;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
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
pub struct PaginatedChannels {
    pub channels: Vec<Channel>,
    pub page_size: u64,
    pub total_count: u64,
}

#[derive(Debug, Parser, Default, PartialEq, Eq)]
pub struct EventOptions {
    #[clap(short = 'e', long = "event", help = "Event name to send")]
    pub name: Option<String>,
    #[clap(short = 'd', long = "data", help = "Event data to send", value_parser = validate_json)]
    pub data: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct MessageEvent {
    #[serde(rename = "e")]
    pub event: String,
    #[serde(rename = "d")]
    pub data: Value,
}
