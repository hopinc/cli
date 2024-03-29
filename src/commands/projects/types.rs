use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

use crate::commands::ignite::types::{Resources, Volume};
use crate::utils::deser::{deserialize_null_default, deserialize_string_to_f64};
use crate::utils::size::{parse_size, unit_multiplier, user_friendly_size};

// types for the API response
#[derive(Debug, Deserialize)]
pub struct ProjectRes {
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub struct SingleProjectResponse {
    pub project: Project,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub enum ProjectTier {
    #[default]
    #[serde(rename = "free")]
    Free,
    #[serde(rename = "paid")]
    Paid,
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
    //@this route does not return this field at all, hence the serde(default)
    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub tier: ProjectTier,
}

impl Project {
    pub fn is_personal(&self) -> bool {
        self.type_ == "personal"
    }

    pub fn is_paid(&self) -> bool {
        matches!(self.tier, ProjectTier::Paid)
    }
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
#[serde(default)] // some quotas like volume can be missing in overrides, so it's better to
                  // default to 0
pub struct Quota {
    pub vcpu: f64,
    pub ram: u64,
    pub volume: u64,
}

#[derive(Debug, Deserialize, Default)]
pub struct Quotas {
    #[serde(rename = "default_quotas")]
    pub default: Quota,
    #[serde(rename = "quota_overrides")]
    pub overrides: Quota,
    #[serde(rename = "quota_usage")]
    pub usage: Quota,
}

impl std::ops::Sub for Quota {
    type Output = Quota;

    fn sub(self, rhs: Self) -> Self::Output {
        Quota {
            vcpu: self.vcpu - rhs.vcpu,
            ram: self.ram - rhs.ram,
            volume: self.volume - rhs.volume,
        }
    }
}

impl Quotas {
    pub fn total_quota(&self) -> Quota {
        Quota {
            vcpu: if self.overrides.vcpu > 0f64 {
                self.overrides.vcpu
            } else {
                self.default.vcpu
            },
            ram: (if self.overrides.ram > 0 {
                self.overrides.ram
            } else {
                self.default.ram
            }) * unit_multiplier::MB,
            volume: (if self.overrides.volume > 0 {
                self.overrides.volume
            } else {
                self.default.volume
            }) * unit_multiplier::MB,
        }
    }

    pub fn usage_quota(&self) -> Quota {
        Quota {
            vcpu: self.usage.vcpu,
            ram: self.usage.ram * unit_multiplier::MB,
            volume: self.usage.volume * unit_multiplier::MB,
        }
    }

    pub fn free_quota(&self) -> Quota {
        Quota {
            vcpu: self.default.vcpu,
            ram: self.default.ram * unit_multiplier::MB,
            volume: self.default.volume * unit_multiplier::MB,
        }
    }

    pub fn can_deploy(
        &self,
        resources: &Resources,
        volume: &Option<Volume>,
        project: &Project,
    ) -> Result<()> {
        let total = self.total_quota();
        let usage = self.usage_quota();

        let error = if project.is_paid() {
            "Please contact us to increase your quota."
        } else {
            "Please attach a payment method to your project."
        };

        if usage.vcpu + resources.vcpu > total.vcpu {
            bail!(
                "Not enough vCPU quota, you need additional {} vCPU. {error}",
                usage.vcpu + resources.vcpu - total.vcpu,
            );
        }

        let ram = parse_size(&resources.ram)?;

        if usage.ram + ram > total.ram {
            bail!(
                "Not enough Memory quota, you need additional {}'s of RAM. {error}",
                user_friendly_size(usage.ram + ram - total.ram)?
            );
        }

        if let Some(volume) = volume {
            let volume = parse_size(&volume.size)?;

            if usage.volume + volume > total.volume {
                bail!(
                    "Not enough Volume quota, you need additional {}'s of storage. {error}",
                    user_friendly_size(usage.volume + volume - total.volume)?
                );
            }
        }

        Ok(())
    }

    pub fn get_free_tier_billable(
        &self,
        resources: &Resources,
        volume: &Option<Volume>,
    ) -> Result<(bool, (Resources, Option<String>))> {
        let mut free_tier_applicable = false;
        let mut billable_resources = Resources::default();
        let mut billable_volume = None;

        let left_free = self.free_quota() - self.usage_quota();

        if left_free.vcpu > 0f64 {
            free_tier_applicable = true;
            billable_resources.vcpu = if resources.vcpu > left_free.vcpu {
                resources.vcpu - left_free.vcpu
            } else {
                0f64
            };
        } else {
            billable_resources.vcpu = resources.vcpu;
        }

        if left_free.ram > 0 {
            let ram = parse_size(&resources.ram)?;

            free_tier_applicable = true;
            billable_resources.ram = format!(
                "{}B",
                if ram > left_free.ram {
                    ram - left_free.ram
                } else {
                    0
                }
            );

            log::debug!("{} {}", left_free.ram, ram);
        } else {
            billable_resources.ram = resources.ram.clone();
        }

        if let Some(volume) = volume {
            if left_free.volume > 0 {
                let volume = parse_size(&volume.size)?;

                free_tier_applicable = true;
                billable_volume = Some(format!(
                    "{}B",
                    if volume > left_free.volume {
                        volume - left_free.volume
                    } else {
                        0
                    }
                ));
            } else {
                billable_volume = Some(volume.size.clone());
            }
        }

        log::debug!(
            "free_tier_applicable: {}, billable_resources: {:?}, billable_volume: {:?}",
            free_tier_applicable,
            billable_resources,
            billable_volume
        );

        Ok((free_tier_applicable, (billable_resources, billable_volume)))
    }
}

#[derive(Debug, Deserialize)]
pub struct Sku {
    pub id: String,
    pub product: String,
    #[serde(deserialize_with = "deserialize_string_to_f64")]
    pub price: f64,
}

#[derive(Debug, Deserialize)]
pub struct SkuResponse {
    pub skus: Vec<Sku>,
}
