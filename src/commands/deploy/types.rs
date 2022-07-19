use std::str::FromStr;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::util::parse_key_val;

#[derive(Debug, Deserialize)]
pub struct Data {
    pub d: Option<String>,
    pub e: String,
}

#[derive(Debug, Deserialize)]
pub struct Message {
    pub d: Value,
    pub e: String,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Env(pub String, pub String);

impl FromStr for Env {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match parse_key_val(s) {
            Ok((key, val)) => Ok(Env(key, val)),
            Err(e) => Err(format!("Could not pase env value: {}", e)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ContainerOptions {
    pub containers: Option<u64>,
    pub min_containers: Option<u64>,
    pub max_containers: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct CreateContainers {
    pub count: u64,
}
