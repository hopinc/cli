use std::collections::HashMap;
use std::fmt::Display;
use std::str::FromStr;
use std::vec;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::commands::containers::types::ContainerType;
use crate::utils::parse_key_val;
use crate::utils::size::{parse_size, unit_multiplier, user_friendly_size};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct Vgpu {
    #[serde(rename = "type")]
    pub type_: String,
    pub count: u32,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, Default)]
#[repr(u32)]
pub enum RamSizes {
    #[serde(rename = "128M")]
    M128,
    #[default]
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
        serde_json::from_str(&format!("\"{}\"", s.replace('B', "").to_uppercase()))
            .map_err(|e| anyhow!(e))
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

impl Resources {
    pub fn get_tier_name(&self, tiers: &[Tier]) -> Result<String> {
        for tier in tiers {
            if tier.resources.cpu == self.vcpu && tier.resources.memory == parse_size(&self.ram)? {
                return Ok(format!(
                    "{} - {} vcpu {}",
                    tier.name,
                    tier.resources.cpu,
                    user_friendly_size(tier.resources.memory)?
                ));
            }
        }

        Ok(format!("{} vcpu {}", self.vcpu, self.ram))
    }
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

impl From<TierResources> for Resources {
    fn from(tier: TierResources) -> Self {
        Self {
            vcpu: tier.cpu,
            ram: format!("{}M", tier.memory),
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
        vec![Self::Manual]
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Default, PartialEq, Eq)]
pub struct Image {
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
#[serde(default)]
pub struct Config {
    pub version: String,
    #[serde(rename = "type")]
    pub type_: ContainerType,
    pub image: Image,
    pub env: HashMap<String, String>,
    pub container_strategy: ScalingStrategy,
    pub resources: Resources,
    pub restart_policy: Option<RestartPolicy>,
    pub entrypoint: Option<Vec<String>>,
    pub cmd: Option<Vec<String>>,
    pub volume: Option<Volume>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
#[serde(default)]
pub struct Deployment {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub container_count: u64,
    pub target_container_count: u64,
    pub config: Config,
    pub group_id: Option<String>,
    #[serde(skip_serializing)]
    pub metadata: Option<Metadata>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, Default)]
pub struct Metadata {
    pub container_port_mappings: Option<HashMap<String, Vec<String>>>,
}

impl Deployment {
    pub fn is_ephemeral(&self) -> bool {
        self.config.type_ == ContainerType::Ephemeral
    }

    pub fn can_rollout(&self) -> bool {
        self.container_count != 0 && self.config.type_ != ContainerType::Stateful
    }

    pub fn can_scale(&self) -> bool {
        self.config.container_strategy == ScalingStrategy::Manual
            && self.config.type_ != ContainerType::Stateful
    }

    pub fn is_stateful(&self) -> bool {
        self.config.type_ == ContainerType::Stateful
    }
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
    pub restart_policy: Option<RestartPolicy>,
    pub container_strategy: ScalingStrategy,
    pub env: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<Image>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub resources: Resources,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub type_: Option<ContainerType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume: Option<Volume>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entrypoint: Option<Vec<String>>,
    #[serde(rename = "cmd", skip_serializing_if = "Option::is_none")]
    pub command: Option<Vec<String>>,
}

impl From<Deployment> for CreateDeployment {
    fn from(deployment: Deployment) -> Self {
        Self {
            restart_policy: deployment.config.restart_policy,
            container_strategy: deployment.config.container_strategy,
            env: deployment.config.env,
            image: Some(deployment.config.image),
            name: Some(deployment.name),
            resources: deployment.config.resources,
            type_: Some(deployment.config.type_),
            volume: deployment.config.volume,
            entrypoint: deployment.config.entrypoint,
            command: deployment.config.cmd,
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

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct Volume {
    pub fs: VolumeFs,
    #[serde(rename = "mountpath")]
    pub mount_path: String,
    pub size: String,
}

impl Default for Volume {
    fn default() -> Self {
        Self {
            fs: VolumeFs::default(),
            mount_path: "/data".to_string(),
            size: "3G".to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum VolumeFs {
    #[default]
    Ext4,
    Xfs,
}

impl FromStr for VolumeFs {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        serde_json::from_str(&format!("\"{}\"", s.to_lowercase())).map_err(|e| anyhow!(e))
    }
}

impl Display for VolumeFs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self).unwrap().replace('"', "")
        )
    }
}

impl VolumeFs {
    pub fn values() -> Vec<Self> {
        vec![Self::Ext4, Self::Xfs]
    }
}

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
pub struct TierResources {
    pub cpu: f64,
    pub memory: u64,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct Tier {
    pub name: String,
    pub description: String,
    pub resources: TierResources,
}

impl Display for Tier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.resources == TierResources::default() {
            write!(f, "{} - {}", self.name, self.description)
        } else {
            write!(
                f,
                "{} - {} ({} CPU, {}MB ram)",
                self.name, self.description, self.resources.cpu, self.resources.memory
            )
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Tiers {
    pub tiers: Vec<Tier>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Premades {
    pub premade: Vec<Premade>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Premade {
    pub name: String,
    pub description: String,
    // #[serde(skip)]
    // pub icon: String,
    pub form: Option<PremadeForm>,
    pub image: String,
    pub entrypoint: Option<Vec<String>>,
    pub mountpath: String,
    pub filesystem: Option<VolumeFs>,
    pub final_note: Option<String>,
    pub environment: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PremadeForm {
    pub v: u64,
    pub fields: Vec<PremadeField>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PremadeField {
    pub title: String,
    pub description: Option<String>,
    #[serde(default)]
    pub required: bool,
    pub input: PremadeInput,
    pub map_to: Vec<MapTo>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum PremadeInput {
    String {
        default: Option<String>,
        autogen: Option<Autogen>,
        max_length: usize,
        validator: String,
        #[serde(default)]
        required: bool,
    },
    Range {
        default: u64,
        min: u64,
        max: u64,
        increment: u64,
        unit: String,
    },
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Autogen {
    ProjectNamespace,
    SecureToken,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MapTo {
    Env { key: String },
    VolumeSize,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RolloutResponse {
    pub rollout: RolloutEvent,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "e", content = "d", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RolloutEvents {
    RolloutCreate(RolloutResponse),
    RolloutUpdate(RolloutEvent),
}

#[derive(Debug, Deserialize, Clone)]
pub struct RolloutEvent {
    pub id: String,
    pub state: RolloutState,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum RolloutState {
    Pending,
    Finished,
    Failed,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Storage {
    pub volume: Option<StorageUsage>,
    pub build_cache: Option<StorageUsage>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StorageUsage {
    pub provisioned_size: u64,
    pub used_size: u64,
}

impl Display for StorageUsage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} / {}",
            get_size(self.used_size),
            get_size(self.provisioned_size)
        )
    }
}

/// Get the size in human readable format
/// size is in megabytes
/// e.g. 1024 -> 1GB
///     512 -> 512MB
///    1 -> 1MB
fn get_size(size: u64) -> String {
    match size {
        1..=unit_multiplier::KB => format!("{}MB", size),

        _ => {
            format!("{:.2}GB", size as f64 / unit_multiplier::KB as f64)
        }
    }
}
