use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

use crate::{
    commands::ignite::types::{Resources, Volume},
    utils::size::parse_size,
};

// types for the API response
#[derive(Debug, Deserialize)]
pub struct ProjectRes {
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub struct SingleProjectResponse {
    pub project: Project,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub icon: Option<String>,
    pub namespace: String,
    #[serde(rename = "type")]
    pub type_: String,
}

#[derive(Debug, Deserialize)]
pub struct ThisProjectResponse {
    pub leap_token: String,
    pub project: Project,
}

#[derive(Debug, Serialize)]
pub struct CreateProject {
    pub name: String,
    pub namespace: String,
    pub payment_method_id: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct Quota {
    pub vcpu: f64,
    pub ram: u64,
    pub volume: u64,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)] // some quotas like volume can be missing in overrides, so it's better to default to 0
pub struct Quotas {
    #[serde(rename = "default_quota")]
    pub default: Quota,
    #[serde(rename = "quota_overrides")]
    pub overrides: Quota,
    #[serde(rename = "quota_usage")]
    pub usage: Quota,
}

impl Quotas {
    pub fn get_vcpu(&self) -> f64 {
        if self.overrides.vcpu > 0f64 {
            self.overrides.vcpu
        } else {
            self.default.vcpu
        }
    }

    pub fn get_ram(&self) -> u64 {
        if self.overrides.ram > 0 {
            self.overrides.ram
        } else {
            self.default.ram
        }
    }

    pub fn get_volume(&self) -> u64 {
        if self.overrides.volume > 0 {
            self.overrides.volume
        } else {
            self.default.volume
        }
    }

    pub fn can_deploy(&self, resources: &Resources, volume: &Option<Volume>) -> Result<()> {
        if self.usage.vcpu + resources.vcpu > self.get_vcpu() {
            bail!(
                "Not enough vCPU quota, you need additional {} vCPU. Please contact support.",
                self.usage.vcpu + resources.vcpu - self.get_vcpu()
            );
        }

        let ram = parse_size(&resources.ram)?;

        if self.usage.ram + ram > self.get_ram() {
            bail!(
                "Not enough RAM quota, you need additional {}B RAM. Please contact support.",
                self.usage.ram + ram - self.get_ram()
            );
        }

        if let Some(volume) = volume {
            let volume = parse_size(&volume.size)?;

            if self.usage.volume + volume > self.get_volume() {
                bail!(
                    "Not enough volume quota, you need additional {}B volume. Please contact support.",
                    self.usage.volume + volume- self.get_volume()
                );
            }
        }

        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct Sku {
    pub id: String,
    pub product: String,
    #[serde(deserialize_with = "de_string_to_f64")]
    pub price: f64,
}

fn de_string_to_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    String::deserialize(deserializer)?
        .parse::<f64>()
        .map_err(serde::de::Error::custom)
}
