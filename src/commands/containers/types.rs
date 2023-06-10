use std::fmt::Display;
use std::str::FromStr;

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

#[derive(Debug, Deserialize, Clone)]
pub struct Uptime {
    pub last_start: Option<DateTime<Utc>>,
}
#[derive(Debug, Deserialize, Clone)]
pub struct Container {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub state: ContainerState,
    pub metrics: Option<Metrics>,
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

#[derive(Debug, Deserialize)]
pub struct SingleContainer {
    pub container: Container,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Metrics {
    pub cpu_usage_percent: f64,
    pub memory_usage_bytes: u64,
}

/// Reusable metrics functions
impl Metrics {
    /// Normalize the metrics to the number of vcpus
    pub fn cpu_usage_percent(&self, cpu_count: f64) -> f64 {
        // 100% = 4vcpu
        self.cpu_usage_percent / cpu_count / 4.0
    }

    /// Normalize the metrics to the amount of memory
    pub fn memory_usage_percent(&self, memory: u64) -> f64 {
        self.memory_usage_bytes as f64 / (memory as f64) * 100.0
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "e", content = "d", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ContainerEvents {
    ContainerMetricsUpdate {
        container_id: String,
        metrics: Metrics,
    },
}
