use anyhow::{anyhow, ensure, Result};
use clap::Parser;

use super::create::Options as CreateOptions;
use crate::commands::ignite::types::{MultipleDeployments, ScalingStrategy};
use crate::commands::ignite::util::{rollout, scale, update_deployment, update_deployment_config};
use crate::state::State;

#[derive(Debug, Parser)]
#[clap(about = "Update a deployment")]
pub struct Options {
    #[clap(name = "deployment", help = "NAME or ID of the deployment to update")]
    deployment: Option<String>,

    #[clap(flatten)]
    config: CreateOptions,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    let project_id = state.ctx.current_project_error().id;

    let deployments = state
        .http
        .request::<MultipleDeployments>(
            "GET",
            &format!("/ignite/deployments?project={}", project_id),
            None,
        )
        .await?
        .ok_or_else(|| anyhow!("Failed to parse response"))?
        .deployments;

    ensure!(!deployments.is_empty(), "No deployments found");

    let old_deployment = match options.deployment {
        Some(name_or_id) => {
            let deployment = deployments
                .iter()
                .find(|p| p.name == name_or_id || p.id == name_or_id)
                .expect("Deployment not found");

            log::info!(
                "Updating deployment `{}` ({})",
                deployment.name,
                deployment.id
            );

            deployment.clone()
        }
        None => {
            let deployments_fmt = deployments
                .iter()
                .map(|d| format!("{} ({})", d.name, d.id))
                .collect::<Vec<_>>();

            let idx = dialoguer::Select::new()
                .with_prompt("Select a deployment to update")
                .items(&deployments_fmt)
                .default(0)
                .interact_opt()
                .expect("Failed to select deployment")
                .expect("No deployment selected");

            deployments[idx].clone()
        }
    };

    let (deployment_config, container_options) = update_deployment_config(
        options.config.clone(),
        options.config != CreateOptions::default(),
        &old_deployment,
        &None,
    );

    let deployment = update_deployment(
        &state.http,
        &project_id,
        &old_deployment.id,
        &deployment_config,
    )
    .await
    .map_err(|e| anyhow!("Failed to update deployment: {}", e))?;

    if deployment.container_count > 0 {
        log::info!("Rolling out new containers");
        rollout(&state.http, &deployment.id).await?;
    }

    if deployment.config.container_strategy == ScalingStrategy::Manual {
        if let Some(count) = container_options.containers {
            log::info!(
                "Updating container count from {} to {}",
                old_deployment.container_count,
                count
            );

            scale(&state.http, &deployment.id, count).await?;
        }
    }

    log::info!(
        "Deployment `{}` ({}) updated",
        deployment.name,
        deployment.id
    );

    Ok(())
}
