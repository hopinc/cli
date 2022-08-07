use std::collections::HashMap;
use std::str::FromStr;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use super::util::parse_key_val;
use crate::commands::containers::types::ContainerType;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct Vgpu {
    #[serde(rename = "type")]
    pub g_type: String,
    pub count: u32,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
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
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        serde_json::from_str(&format!("\"{}\"", s.to_uppercase())).map_err(|e| anyhow!(e))
    }
}

impl ToString for RamSizes {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap().replace('"', "")
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

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Resources {
    pub vcpu: f64,
    pub ram: String,
    #[serde(skip)]
    pub vgpu: Vec<Vgpu>,
}

impl Default for Resources {
    fn default() -> Self {
        Resources {
            vcpu: 0.5,
            ram: RamSizes::default().to_string(),
            vgpu: vec![],
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub enum ScalingStrategy {
    #[serde(rename = "manual")]
    Manual,
    #[serde(rename = "autoscale")]
    Autoscaled,
}

impl FromStr for ScalingStrategy {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        serde_json::from_str(&format!("\"{}\"", s.to_lowercase())).map_err(|e| anyhow!(e))
    }
}

impl ToString for ScalingStrategy {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap().replace('"', "")
    }
}

impl Default for ScalingStrategy {
    fn default() -> Self {
        ScalingStrategy::Autoscaled
    }
}

impl ScalingStrategy {
    pub fn values() -> Vec<ScalingStrategy> {
        vec![ScalingStrategy::Manual, ScalingStrategy::Autoscaled]
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Default, PartialEq, Eq)]
pub struct Image {
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
pub struct Config {
    pub version: String,
    #[serde(rename = "type")]
    pub d_type: ContainerType,
    pub image: Image,
    pub env: HashMap<String, String>,
    pub container_strategy: ScalingStrategy,
    pub resources: Resources,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
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

#[derive(Debug, Serialize, Clone, Default, PartialEq)]
pub struct CreateDeployment {
    pub container_strategy: ScalingStrategy,
    pub env: HashMap<String, String>,
    pub image: Image,
    pub name: String,
    pub resources: Resources,
    #[serde(rename = "type")]
    pub container_type: ContainerType,
}

impl CreateDeployment {
    pub fn from_deployment(deployment: &Deployment) -> Self {
        Self {
            container_strategy: deployment.config.container_strategy.clone(),
            env: deployment.config.env.clone(),
            image: deployment.config.image.clone(),
            name: deployment.name.clone(),
            resources: deployment.config.resources.clone(),
            container_type: deployment.config.d_type.clone(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Env(pub String, pub String);

impl FromStr for Env {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match parse_key_val(s) {
            Ok((key, val)) => Ok(Env(key, val)),
            Err(e) => Err(anyhow!("Could not pase env value: {}", e)),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ScaleRequest {
    pub scale: u64,
}
