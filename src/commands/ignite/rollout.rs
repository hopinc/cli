use anyhow::Result;
use clap::Parser;

use super::util::{format_deployments, get_all_deployments, rollout};
use crate::state::State;

#[derive(Debug, Parser)]
#[clap(about = "Rollout new containers to a deployment")]
pub struct Options {
    #[clap(name = "deployment", help = "NAME or ID of the deployment to rollout")]
    pub deployment: Option<String>,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    let deployment = match options.deployment {
        Some(deployment) => {
            if deployment.starts_with("deployment_") {
                deployment
            } else {
                let project = state.ctx.current_project_error();

                log::info!("Using deployment {} /{}", project.name, project.namespace);

                let deployments = get_all_deployments(&state.http, &project.id).await?;

                deployments
                    .iter()
                    .find(|d| d.name == deployment)
                    .map(|d| d.id.clone())
                    .expect("Deployment not found")
            }
        }
        None => {
            let project = state.ctx.current_project_error();

            log::info!("Using deployment {} /{}", project.name, project.namespace);

            let deployments = get_all_deployments(&state.http, &project.id).await?;

            let deployments_fmt = format_deployments(&deployments, false);

            let idx = dialoguer::Select::new()
                .with_prompt("Select a deployment")
                .items(&deployments_fmt)
                .default(0)
                .interact_opt()
                .expect("Failed to select a deployment")
                .expect("Failed to select a deployment");

            deployments[idx].id.clone()
        }
    };

    rollout(&state.http, &deployment).await?;

    log::info!("Rollling out new containers");

    Ok(())
}