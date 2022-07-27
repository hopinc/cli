use serde_json::Value;

use super::types::{Container, CreateContainers, CreateContainersResponse};
use crate::state::http::HttpClient;

pub async fn create_containers(
    http: HttpClient,
    deployment_id: String,
    count: u64,
) -> Vec<Container> {
    http.request::<CreateContainersResponse>(
        "POST",
        format!("/ignite/deployments/{}/containers", deployment_id).as_str(),
        Some((
            serde_json::to_string(&CreateContainers { count })
                .unwrap()
                .into(),
            "application/json",
        )),
    )
    .await
    .expect("Failed to create containers")
    .expect("Failed to create containers")
    .containers
}

pub async fn rollout(http: HttpClient, deployment_id: String) {
    http.request::<Value>(
        "POST",
        format!("/ignite/deployments/{}/rollouts", deployment_id).as_str(),
        None,
    )
    .await
    .expect("Failed to rollout")
    .expect("Failed to rollout");
}
