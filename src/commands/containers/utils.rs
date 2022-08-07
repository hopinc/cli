use super::types::{Container, CreateContainers, CreateContainersResponse};
use crate::state::http::HttpClient;

pub async fn create_containers(
    http: &HttpClient,
    deployment_id: &str,
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
