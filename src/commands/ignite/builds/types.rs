use std::fmt::Display;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct MultipleBuilds {
    pub builds: Vec<Build>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum BuildMethod {
    Cli,
    Github,
}

impl Display for BuildMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self).unwrap().replace('"', "")
        )
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum BuildState {
    Pending,
    Succeeded,
    Failed,
    Cancelled,
}

impl Display for BuildState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self).unwrap().replace('"', "")
        )
    }
}

#[derive(Debug, Deserialize)]
pub struct Build {
    pub id: String,
    pub deployment_id: String,
    pub method: BuildMethod,
    pub started_at: DateTime<Utc>,
    pub state: BuildState,
    pub digest: Option<String>,
    pub finished_at: Option<DateTime<Utc>>,
}
