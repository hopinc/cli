use serde_derive::Deserialize;
use serde_derive::Serialize;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Root {
    pub name: Option<String>,
    pub version: Option<String>,
    pub secrets: Option<std::collections::HashMap<String, Secret>>,
    pub configs: Option<std::collections::HashMap<String, Config>>,
    pub services: Option<std::collections::HashMap<String, Service>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConfigExternalUnion {
    Bool(bool),
    Named(ConfigExternalNamed),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfigExternalNamed {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Secret {
    pub driver: Option<String>,
    pub external: Option<bool>,
    pub name: Option<String>,

    // Unknown value
    pub labels: Option<std::collections::HashMap<String, Value>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub labels: Option<Vec<String>>,
    pub template_driver: Option<String>,
    pub external: Option<ConfigExternalUnion>,
    pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Service {
    pub credential_spec: Option<CredentialSpec>,
    pub memswap_limit: Option<String>,
    pub pids_limit: Option<f64>,
    pub expose: Option<(String, f64)>,
    pub cgroup_parent: Option<String>,
    pub group_add: Option<(f64, String, String)>,
    pub mem_reservation: Option<i64>,
    pub stdin_open: Option<bool>,
    pub cpu_period: Option<f64>,
    pub init: Option<bool>,
    pub cpus: Option<String>,
    pub mem_limit: Option<f64>,
    pub external_links: Option<Vec<String>>,
    pub runtime: Option<String>,
    pub build: Option<String>,
    pub depends_on: Option<Vec<String>>,
    pub hostname: Option<String>,
    pub isolation: Option<String>,
    pub cpu_count: Option<i64>,
    pub deploy: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CredentialSpec {
    pub registry: Option<String>,
    pub file: Option<String>,
    pub config: Option<String>,
}
