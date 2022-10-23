use std::collections::HashMap;
use std::fmt::Display;
use std::str::FromStr;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use super::utils::parse_key_val;
use crate::commands::containers::types::ContainerType;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct Vgpu {
    #[serde(rename = "type")]
    pub type_: String,
    pub count: u32,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, Default)]
#[repr(u32)]
pub enum RamSizes {
    #[default]
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

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, Default)]
pub enum ScalingStrategy {
    #[default]
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

impl Display for ScalingStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self).unwrap().replace('"', "")
        )
    }
}

impl ScalingStrategy {
    pub fn values() -> Vec<Self> {
        vec![Self::Manual, Self::Autoscaled]
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
    pub type_: ContainerType,
    pub image: Image,
    pub env: HashMap<String, String>,
    pub container_strategy: ScalingStrategy,
    pub resources: Resources,

    #[serde(default)]
    pub restart_policy: RestartPolicy,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
pub struct Deployment {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub container_count: u64,
    pub target_container_count: u64,
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
    pub restart_policy: RestartPolicy,
    pub container_strategy: ScalingStrategy,
    pub env: HashMap<String, String>,
    pub image: Image,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub resources: Resources,
    #[serde(rename = "type")]
    pub type_: ContainerType,
}

impl CreateDeployment {
    pub fn from_deployment(deployment: &Deployment) -> Self {
        Self {
            restart_policy: deployment.config.restart_policy.clone(),
            container_strategy: deployment.config.container_strategy.clone(),
            env: deployment.config.env.clone(),
            image: deployment.config.image.clone(),
            name: Some(deployment.name.clone()),
            resources: deployment.config.resources.clone(),
            type_: deployment.config.type_.clone(),
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

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, Default)]
pub enum RestartPolicy {
    #[serde(rename = "never")]
    Never,
    #[serde(rename = "always")]
    Always,
    #[default]
    #[serde(rename = "on-failure")]
    OnFailure,
}

impl FromStr for RestartPolicy {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        serde_json::from_str(&format!("\"{}\"", s.to_lowercase())).map_err(|e| anyhow!(e))
    }
}

impl Display for RestartPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self).unwrap().replace('"', "")
        )
    }
}

impl RestartPolicy {
    pub fn values() -> Vec<Self> {
        vec![Self::Never, Self::Always, Self::OnFailure]
    }
}
