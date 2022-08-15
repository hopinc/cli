use std::{fmt::Display, str::FromStr};

use anyhow::anyhow;
use serde::{Deserialize, Serialize};

use crate::commands::domains::types::Domain;

#[derive(Debug, Serialize, Clone, Default, PartialEq)]
pub struct GatewayConfig {
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub type_: Option<GatewayType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol: Option<GatewayProtocol>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub internal_domain: Option<String>,
}

impl GatewayConfig {
    pub fn from_gateway(gateway: &Gateway) -> Self {
        Self {
            type_: Some(gateway.type_.clone()),
            protocol: gateway.protocol.clone(),
            name: gateway.name.clone(),
            target_port: gateway.target_port,
            internal_domain: gateway.internal_domain.clone(),
        }
    }
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct Gateway {
    pub id: String,
    pub created_at: String,
    pub hopsh_domain: Option<String>,
    pub internal_domain: Option<String>,
    pub name: Option<String>,
    pub protocol: Option<GatewayProtocol>,
    pub target_port: Option<u16>,
    #[serde(rename = "type")]
    pub type_: GatewayType,
    #[serde(default)]
    pub domains: Vec<Domain>,
}

#[derive(Debug, Deserialize)]
pub struct SingleGateway {
    pub gateway: Gateway,
}

#[derive(Debug, Deserialize)]
pub struct MultipleGateways {
    pub gateways: Vec<Gateway>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default, PartialEq)]
pub enum GatewayType {
    #[default]
    #[serde(rename = "external")]
    External,
    #[serde(rename = "internal")]
    Internal,
}

impl FromStr for GatewayType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(&format!("\"{}\"", s.to_lowercase())).map_err(|e| anyhow!(e))
    }
}

impl Display for GatewayType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self).unwrap().replace('"', "")
        )
    }
}

impl GatewayType {
    pub fn values() -> Vec<GatewayType> {
        vec![GatewayType::External, GatewayType::Internal]
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum GatewayProtocol {
    #[serde(rename = "http")]
    Http,
}

impl FromStr for GatewayProtocol {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(&format!("\"{}\"", s.to_lowercase())).map_err(|e| anyhow!(e))
    }
}

impl Display for GatewayProtocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self).unwrap().replace('"', "")
        )
    }
}

impl Default for GatewayProtocol {
    fn default() -> Self {
        GatewayProtocol::Http
    }
}

impl GatewayProtocol {
    pub fn values() -> Vec<GatewayProtocol> {
        vec![GatewayProtocol::Http]
    }
}
