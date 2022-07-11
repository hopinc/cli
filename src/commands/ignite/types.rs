use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Vgpu {
    #[serde(rename = "type")]
    pub g_type: String,
    pub count: u32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Resources {
    pub cpu: u64,
    pub ram: String,
    #[serde(skip)]
    pub vgpu: Vec<Vgpu>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum ContainerStrategy {
    #[serde(rename = "manual")]
    Manual,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Image {
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum ContainerType {
    #[serde(rename = "ephemeral")]
    Ephemeral,
    #[serde(rename = "persistent")]
    Persistent,
}

impl FromStr for ContainerType {
    type Err = String;
    fn from_str(day: &str) -> Result<Self, Self::Err> {
        match day {
            "ephemeral" => Ok(ContainerType::Ephemeral),
            "persistent" => Ok(ContainerType::Persistent),
            _ => Err("Invalid container type".to_string()),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub version: String,
    #[serde(rename = "type")]
    pub d_type: ContainerType,
    pub image: Image,
    pub container_strategy: ContainerStrategy,
    pub resources: Resources,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Deployment {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub container_count: u32,
    pub config: Config,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SingleDeployment {
    pub deployment: Deployment,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MultipleDeployments {
    pub deployments: Vec<Deployment>,
}

#[derive(Debug, Serialize, Clone)]
pub struct CreateDeployment {
    pub container_strategy: ContainerStrategy,
    pub env: HashMap<String, String>,
    pub image: Image,
    pub name: String,
    pub resources: Resources,
    #[serde(rename = "type")]
    pub d_type: ContainerType,
}
