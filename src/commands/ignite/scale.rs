use anyhow::{ensure, Result};
use clap::Parser;

use super::utils::{format_deployments, get_all_deployments, scale};
use crate::commands::ignite::utils::get_deployment;
use crate::state::State;

#[derive(Debug, Parser)]
#[clap(about = "Scale a deployment")]
pub struct Options {
    #[clap(help = "ID of the deployment to scale")]
    pub deployment: Option<String>,

    #[clap(help = "Number of replicas to scale to")]
    pub scale: Option<u64>,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    let deployment = match options.deployment {
        Some(id) => get_deployment(&state.http, &id).await?,

        None => {
            let project_id = state.ctx.current_project_error().id;

            let deployments = get_all_deployments(&state.http, &project_id).await?;
            ensure!(!deployments.is_empty(), "No deployments found");
            let deployments_fmt = format_deployments(&deployments, false);

            let idx = dialoguer::Select::new()
                .with_prompt("Select a deployment")
                .items(&deployments_fmt)
                .default(0)
                .interact_opt()
                .expect("Failed to select deployment")
                .expect("No deployment selected");

            deployments[idx].clone()
        }
    };

    let scale_count = match options.scale {
        Some(scale) => scale,
        None => dialoguer::Input::<u64>::new()
            .with_prompt("Enter the number of containers to scale to")
            .default(deployment.container_count)
            .interact()
            .expect("Failed to select a deployment"),
    };

    scale(&state.http, &deployment.id, scale_count).await?;

    log::info!("Scaling deployment to {} containers", scale_count);

    Ok(())
}
