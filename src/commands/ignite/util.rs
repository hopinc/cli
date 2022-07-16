use super::types::{Deployment, MultipleDeployments};
use crate::state::http::HttpClient;

pub async fn get_deployments(http: HttpClient, project_id: String) -> Vec<Deployment> {
    http.request::<MultipleDeployments>(
        "GET",
        &format!("/ignite/deployments?project={}", project_id),
        None,
    )
    .await
    .expect("Error while getting deployments")
    .unwrap()
    .deployments
}

pub fn format_deployments(deployments: &Vec<Deployment>) -> Vec<String> {
    deployments
        .iter()
        .map(|d| {
            format!(
                " {} ({}) - {} container{}",
                d.name,
                d.id,
                d.container_count,
                if d.container_count == 1 { "" } else { "s" }
            )
        })
        .collect::<Vec<_>>()
}
