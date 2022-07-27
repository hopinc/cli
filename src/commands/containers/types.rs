use std::str::FromStr;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum ContainerType {
    #[serde(rename = "ephemeral")]
    Ephemeral,
    #[serde(rename = "persistent")]
    Persistent,
}

impl FromStr for ContainerType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(format!("\"{}\"", s.to_lowercase()).as_str())
            .map_err(|e| e.to_string())
    }
}

impl ToString for ContainerType {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap().replace("\"", "")
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

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum ContainerState {
    #[serde(rename = "exited")]
    Exited,
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "running")]
    Running,
}

impl ToString for ContainerState {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap().replace("\"", "")
    }
}

#[derive(Debug, Deserialize)]
pub struct Container {
    pub id: String,
    pub created_at: String,
    pub state: ContainerState,
    pub deployment_id: String,
    pub internal_ip: Option<String>,
    // TODO: types
    pub uptime: Option<Value>,
    #[serde(rename = "type")]
    pub c_type: ContainerType,
}

#[derive(Debug, Deserialize)]

pub struct CreateContainersResponse {
    pub containers: Vec<Container>,
}

#[derive(Debug, Clone)]
pub struct ContainerOptions {
    pub containers: Option<u64>,
    pub min_containers: Option<u64>,
    pub max_containers: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct CreateContainers {
    pub count: u64,
}
