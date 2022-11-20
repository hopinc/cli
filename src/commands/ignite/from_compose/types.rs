use std::collections::HashMap;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_yaml::Value;

use crate::utils::parse_key_val;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DockerCompose {
    pub name: Option<String>,
    pub version: Option<String>,
    pub secrets: Option<HashMap<String, Secret>>,
    pub services: Option<HashMap<String, Service>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConfigExternalUnion {
    Bool(bool),
    Named(ConfigExternalNamed),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigExternalNamed {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Secret {
    pub driver: Option<String>,
    pub external: Option<bool>,
    pub name: Option<String>,

    // Unknown value
    pub labels: Option<HashMap<String, Value>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceBuildMap {
    context: Option<String>,
    dockerfile: Option<String>,
    args: Option<HashMap<String, Value>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ServiceBuildUnion {
    String(String),
    Map(ServiceBuildMap),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Service {
    pub expose: Option<(String, f64)>,
    pub ports: Option<Vec<String>>,
    pub environment: Option<Env>,
    pub restart: Option<String>,
    pub image: Option<String>,
    pub build: Option<ServiceBuildUnion>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Env(HashMap<String, String>);

impl<'de> Deserialize<'de> for Env {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;

        match value {
            Value::Sequence(seq) => {
                let mut map = HashMap::new();

                // log::debug!("Sequence: {:?}", seq);

                for item in seq {
                    let item_str = item
                        .as_str()
                        .context("Failed to parse environment variable from sequence")
                        .map_err(serde::de::Error::custom)?;

                    let (key, value) = parse_key_val::<String, String>(item_str)
                        .map_err(|error| serde::de::Error::custom(error.to_string()))?;

                    map.insert(key, value);
                }

                Ok(Env(map))
            }

            Value::Mapping(mapping) => {
                let mut map = HashMap::new();

                // log::debug!("Mapping: {:?}", mapping);

                for (key, value) in mapping {
                    let key = key
                        .as_str()
                        .context("Failed to parse environment variable")
                        .map_err(serde::de::Error::custom)?;

                    let value = value
                        .as_str()
                        .map(|s| s.to_string())
                        .or_else(|| value.as_f64().map(|f| f.to_string()))
                        .context("INvalid value in mapping, expected string or number")
                        .map_err(serde::de::Error::custom)?;

                    map.insert(key.to_string(), value.to_string());
                }

                Ok(Env(map))
            }

            _ => Err(serde::de::Error::custom(
                "Failed to parse environment variable",
            )),
        }
    }
}
