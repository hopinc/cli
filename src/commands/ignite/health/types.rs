use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct CreateHealthCheck {
    pub initial_delay: u64,
    pub interval: u64,
    pub max_retries: u64,
    pub path: String,
    pub protocol: String,
    pub port: u16,
    pub timeout: u64,
    pub success_threshold: u64,
}

impl Default for CreateHealthCheck {
    fn default() -> Self {
        Self {
            initial_delay: 5,
            interval: 60,
            max_retries: 3,
            path: String::from("/"),
            protocol: String::from("HTTP"),
            port: 8080,
            timeout: 50,
            success_threshold: 1,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthCheckType {
    Liveness,
}

#[derive(Debug, Deserialize)]
pub struct HealthCheck {
    pub id: String,
    pub deployment_id: String,
    pub initial_delay: u64,
    pub interval: u64,
    pub max_retries: u64,
    pub path: String,
    pub protocol: String,
    pub port: u64,
    pub timeout: u64,
    pub success_threshold: u64,
    pub created_at: String,
    #[serde(rename = "type")]
    pub type_: HealthCheckType,
}

#[derive(Debug, Deserialize)]
pub struct SingleHealthCheck {
    pub health_check: HealthCheck,
}

#[derive(Debug, Deserialize)]
pub struct MultipleHealthChecks {
    pub health_checks: Vec<HealthCheck>,
}

#[derive(Debug, Deserialize)]
pub struct HealthCheckState {
    pub state: String,
    pub container_id: String,
    pub health_check_id: String,
    pub deployment_id: String,
    pub created_at: String,
    pub next_check: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct MultipleHealthCheckState {
    pub health_check_states: Vec<HealthCheckState>,
}
