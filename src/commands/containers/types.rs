use std::fmt::Display;
use std::str::FromStr;
use std::vec;

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::commands::ignite::types::Deployment;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ContainerType {
    Ephemeral,
    #[default]
    Persistent,
    Stateful,
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

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ContainerState {
    Exited,
    Pending,
    Running,
    Stopped,
    Terminating,
    Failed,
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

impl ContainerState {
    pub fn from_changeable_state(state: &ChangeableContainerState) -> Self {
        match state {
            ChangeableContainerState::Start => Self::Running,
            ChangeableContainerState::Stop => Self::Stopped,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub enum ChangeableContainerState {
    #[serde(rename = "stop")]
    Stop,

    #[serde(rename = "start")]
    Start,
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
    pub fn values() -> Vec<Self> {
        vec![Self::Stop, Self::Start]
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Uptime {
    pub last_start: Option<DateTime<Utc>>,
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
            containers: Some(deployment.container_count),
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
pub struct UpdateContainerState {
    pub preferred_state: ContainerState,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Log {
    pub timestamp: DateTime<Utc>,
    pub level: String,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct LogsResponse {
    pub logs: Vec<Log>,
}
