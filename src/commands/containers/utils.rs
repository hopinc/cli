use anyhow::{anyhow, Result};

use super::types::{Container, CreateContainers, CreateContainersResponse};
use crate::state::http::HttpClient;

pub async fn create_containers(
    http: &HttpClient,
    deployment_id: &str,
    count: u64,
) -> Result<Vec<Container>> {
    let response = http
        .request::<CreateContainersResponse>(
            "POST",
            &format!("/ignite/deployments/{}/containers", deployment_id),
            Some((
                serde_json::to_string(&CreateContainers { count })
                    .unwrap()
                    .into(),
                "application/json",
            )),
        )
        .await?
        .ok_or_else(|| anyhow!("Error while parsing response"))?;

    Ok(response.containers)
}
