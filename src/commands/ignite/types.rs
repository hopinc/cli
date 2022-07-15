use std::collections::HashMap;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Vgpu {
    #[serde(rename = "type")]
    pub g_type: String,
    pub count: u32,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[repr(u32)]
pub enum RamSizes {
    #[serde(rename = "128M")]
    M128 = 128,
    #[serde(rename = "256M")]
    M256 = 256,
    #[serde(rename = "512M")]
    M512 = 512,
    #[serde(rename = "1G")]
    G1 = 1024,
    #[serde(rename = "2G")]
    G2 = 2048,
    #[serde(rename = "4G")]
    G4 = 4096,
    #[serde(rename = "8G")]
    G8 = 8192,
    #[serde(rename = "16G")]
    G16 = 16384,
    #[serde(rename = "32G")]
    G32 = 32768,
    #[serde(rename = "64G")]
    G64 = 65536,
}

impl FromStr for RamSizes {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "128MB" => Ok(RamSizes::M128),
            "256MB" => Ok(RamSizes::M256),
            "512MB" => Ok(RamSizes::M512),
            "1GB" => Ok(RamSizes::G1),
            "2GB" => Ok(RamSizes::G2),
            "4GB" => Ok(RamSizes::G4),
            "8GB" => Ok(RamSizes::G8),
            "16GB" => Ok(RamSizes::G16),
            "32GB" => Ok(RamSizes::G32),
            "64GB" => Ok(RamSizes::G64),
            _ => Err("Invalid RAM size, has to be one of `128MB`, `256MB`, `512MB`, `1GB`, `2GB`, `4GB`, `8GB`, `16GB`, `32GB`, `64GB`".to_string()),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Resources {
    pub cpu: u64,
    pub ram: RamSizes,
    #[serde(skip)]
    pub vgpu: Vec<Vgpu>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum ScalingStrategy {
    #[serde(rename = "manual")]
    Manual,
    #[serde(rename = "stateful")]
    Stateful,
    #[serde(rename = "autoscaled")]
    Autoscaled,
}

impl FromStr for ScalingStrategy {
    type Err = String;

    fn from_str(day: &str) -> Result<Self, Self::Err> {
        match day {
            "manual" => Ok(ScalingStrategy::Manual),
            "stateful" => Ok(ScalingStrategy::Stateful),
            "autoscaled" => Ok(ScalingStrategy::Autoscaled),
            _ => Err(
                "Invalid scaling strategy, has to be one of `manual` or `stateful` or `autoscaled`"
                    .to_string(),
            ),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Image {
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
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
            _ => Err(
                "Invalid container type, has to be one of `ephemeral` or `persistent`".to_string(),
            ),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub version: String,
    #[serde(rename = "type")]
    pub d_type: ContainerType,
    pub image: Image,
    pub container_strategy: ScalingStrategy,
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
    pub container_strategy: ScalingStrategy,
    pub env: HashMap<String, String>,
    pub image: Image,
    pub name: String,
    pub resources: Resources,
    #[serde(rename = "type")]
    pub container_type: ContainerType,
}
