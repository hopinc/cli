use std::{fmt::Display, str::FromStr};

use crate::utils::deserialize_from_str;
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::commands::ignite::types::Deployment;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub enum ContainerType {
    #[serde(rename = "ephemeral")]
    Ephemeral,
    #[serde(rename = "persistent")]
    Persistent,
}

impl FromStr for ContainerType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        serde_json::from_str(&format!("\"{}\"", s.to_lowercase())).map_err(|e| anyhow!(e))
    }
}

impl Display for ContainerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self).unwrap().replace('"', "")
        )
    }
}

impl Default for ContainerType {
    fn default() -> Self {
        ContainerType::Persistent
    }
}

impl ContainerType {
    pub fn values() -> Vec<ContainerType> {
        vec![ContainerType::Ephemeral, ContainerType::Persistent]
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum ContainerState {
    #[serde(rename = "exited")]
    Exited,
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "running")]
    Running,
    #[serde(rename = "stopped")]
    Stopped,
    #[serde(rename = "terminating")]
    Terminating,
}

impl Display for ContainerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self).unwrap().replace('"', "")
        )
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum ChangeableContainerState {
    #[serde(rename = "stopped")]
    Stopped,

    #[serde(rename = "running")]
    Running,
}

impl Display for ChangeableContainerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self).unwrap().replace('"', "")
        )
    }
}

impl FromStr for ChangeableContainerState {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        serde_json::from_str(&format!("\"{}\"", s.to_lowercase())).map_err(|e| anyhow!(e))
    }
}

impl ChangeableContainerState {
    pub fn values() -> Vec<ChangeableContainerState> {
        vec![
            ChangeableContainerState::Stopped,
            ChangeableContainerState::Running,
        ]
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Uptime {
    #[serde(deserialize_with = "deserialize_from_str")]
    pub last_start: DateTime<Utc>,
}
#[derive(Debug, Deserialize)]
pub struct Container {
    pub id: String,
    pub created_at: String,
    pub state: ContainerState,
    pub deployment_id: String,
    pub internal_ip: Option<String>,
    pub region: String,
    pub uptime: Option<Uptime>,
    #[serde(rename = "type")]
    pub type_: ContainerType,
}

#[derive(Debug, Deserialize)]

pub struct MultipleContainersResponse {
    pub containers: Vec<Container>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ContainerOptions {
    pub containers: Option<u64>,
    pub min_containers: Option<u64>,
    pub max_containers: Option<u64>,
}

impl ContainerOptions {
    pub fn from_deployment(deployment: &Deployment) -> Self {
        Self {
            containers: Some(deployment.container_count as u64),
            min_containers: Some(0),
            max_containers: Some(0),
            // min_containers: Some(deployment.config.resources.min_containers as u64),
            // max_containers: Some(deployment.config.resources.max_containers as u64),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct CreateContainers {
    pub count: u64,
}

#[derive(Debug, Serialize)]
pub struct UpdateContainerState<'a> {
    pub preferred_state: &'a ChangeableContainerState,
}

#[derive(Debug, Deserialize)]
pub struct Log {
    pub nonce: String,
    pub timestamp: DateTime<Utc>,
    pub level: String,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct LogsResponse {
    pub logs: Vec<Log>,
}
