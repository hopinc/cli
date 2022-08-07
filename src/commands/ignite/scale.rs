use anyhow::Result;
use clap::Parser;

use super::util::{format_deployments, get_all_deployments, scale};
use crate::state::State;

#[derive(Debug, Parser)]
#[clap(about = "Scale a deployment")]
pub struct Options {
    #[clap(name = "deployment", help = "NAME or ID of the deployment to scale")]
    pub deployment: Option<String>,

    #[clap(name = "scale", help = "Number of replicas to scale to")]
    pub scale: Option<u64>,
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

    let scale_count = match options.scale {
        Some(scale) => scale,
        None => dialoguer::Input::<u64>::new()
            .with_prompt("Enter the number of containers to scale to")
            .interact()
            .expect("Failed to select a deployment"),
    };

    scale(&state.http, &deployment, scale_count).await?;

    log::info!("Scaling deployment to {} containers", scale_count);

    Ok(())
}
