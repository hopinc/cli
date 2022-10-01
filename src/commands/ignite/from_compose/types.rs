use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DockerCompose {
    pub name: Option<String>,
    pub version: Option<String>,
    pub secrets: Option<std::collections::HashMap<String, Secret>>,
    pub services: Option<std::collections::HashMap<String, Service>>,
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
    pub labels: Option<std::collections::HashMap<String, Value>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceBuildMap {
    context: Option<String>,
    dockerfile: Option<String>,
    args: HashMap<String, Value>,
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
    pub environment: Option<HashMap<String, String>>,
    pub restart: Option<String>,
    pub image: Option<String>,
    pub build: Option<ServiceBuildUnion>,
}
