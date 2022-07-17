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
    M128,
    #[serde(rename = "256M")]
    M256,
    #[serde(rename = "512M")]
    M512,
    #[serde(rename = "1G")]
    G1,
    #[serde(rename = "2G")]
    G2,
    #[serde(rename = "4G")]
    G4,
    #[serde(rename = "8G")]
    G8,
    #[serde(rename = "16G")]
    G16,
    #[serde(rename = "32G")]
    G32,
    #[serde(rename = "64G")]
    G64,
}

impl FromStr for RamSizes {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(format!("\"{}\"", s.to_uppercase()).as_str())
            .map_err(|e| e.to_string())
    }
}

impl ToString for RamSizes {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap().replace("\"", "")
    }
}

impl Default for RamSizes {
    fn default() -> Self {
        RamSizes::M512
    }
}

impl RamSizes {
    pub fn values() -> Vec<RamSizes> {
        vec![
            RamSizes::M128,
            RamSizes::M256,
            RamSizes::M512,
            RamSizes::G1,
            RamSizes::G2,
            RamSizes::G4,
            RamSizes::G8,
            RamSizes::G16,
            RamSizes::G32,
            RamSizes::G64,
        ]
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Resources {
    pub cpu: u64,
    pub ram: String,
    #[serde(skip)]
    pub vgpu: Vec<Vgpu>,
}

impl Default for Resources {
    fn default() -> Self {
        Resources {
            cpu: 1,
            ram: RamSizes::default().to_string(),
            vgpu: vec![],
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum ScalingStrategy {
    #[serde(rename = "manual")]
    Manual,
    #[serde(rename = "autoscaled")]
    Autoscaled,
}

impl FromStr for ScalingStrategy {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(format!("\"{}\"", s.to_uppercase()).as_str())
            .map_err(|e| e.to_string())
    }
}

impl ToString for ScalingStrategy {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap().replace("\"", "")
    }
}

impl Default for ScalingStrategy {
    fn default() -> Self {
        ScalingStrategy::Manual
    }
}

impl ScalingStrategy {
    pub fn values() -> Vec<ScalingStrategy> {
        vec![ScalingStrategy::Manual, ScalingStrategy::Autoscaled]
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Image {
    pub name: String,
}

impl Default for Image {
    fn default() -> Self {
        Image {
            name: String::default(),
        }
    }
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

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(format!("\"{}\"", s.to_uppercase()).as_str())
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

impl Default for CreateDeployment {
    fn default() -> Self {
        Self {
            container_strategy: ScalingStrategy::default(),
            env: HashMap::new(),
            image: Image::default(),
            name: String::default(),
            resources: Resources::default(),
            container_type: ContainerType::default(),
        }
    }
}
