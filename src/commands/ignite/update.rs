use anyhow::{anyhow, Result};
use clap::Parser;

use super::create::Options as CreateOptions;
use crate::commands::ignite::groups::utils::fetch_grouped_deployments;
use crate::commands::ignite::utils::{rollout, scale, update_deployment, update_deployment_config};
use crate::state::State;

#[derive(Debug, Parser)]
#[clap(about = "Update a deployment")]
#[group(skip)]
pub struct Options {
    #[clap(help = "ID of the deployment to update")]
    deployment: Option<String>,

    #[clap(flatten)]
    config: CreateOptions,

    #[clap(long, help = "Do not roll out the changes, only build")]
    no_rollout: bool,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    let project = state.ctx.current_project_error()?;

    let old_deployment = match options.deployment {
        Some(id) => state.get_deployment_by_name_or_id(&id).await?,

        None => {
            let (deployments_fmt, deployments, validator) =
                fetch_grouped_deployments(&state, false, true).await?;

            let idx = loop {
                let idx = dialoguer::Select::new()
                    .with_prompt("Select a deployment")
                    .items(&deployments_fmt)
                    .default(0)
                    .interact()?;

                if let Ok(idx) = validator(idx) {
                    break idx;
                }

                console::Term::stderr().clear_last_lines(1)?
            };

            deployments[idx].clone()
        }
    };

    let is_visual = options.config == CreateOptions::default();

    let (deployment_config, container_options) = update_deployment_config(
        &state.http,
        options.config.clone(),
        is_visual,
        &old_deployment,
        &None,
        true,
        &project,
    )
    .await?;

    let mut deployment = update_deployment(&state.http, &old_deployment.id, &deployment_config)
        .await
        .map_err(|e| anyhow!("Failed to update deployment: {}", e))?;

    if deployment.can_scale() {
        if let Some(count) = container_options.containers {
            log::info!(
                "Updating container count from {} to {}",
                old_deployment.container_count,
                count
            );

            scale(&state.http, &deployment.id, count).await?;

            deployment.container_count = count;
        }
    }

    if deployment.can_rollout() && deployment.container_count > 0 && !options.no_rollout {
        log::info!("Rolling out new containers");
        rollout(&state.http, &deployment.id).await?;
    }

    log::info!(
        "Deployment `{}` ({}) updated",
        deployment.name,
        deployment.id
    );

    Ok(())
}
