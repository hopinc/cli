use anyhow::{ensure, Result};
use clap::Parser;

use super::util::{format_deployments, get_all_deployments, rollout};
use crate::state::State;

#[derive(Debug, Parser)]
#[clap(about = "Rollout new containers to a deployment")]
pub struct Options {
    #[clap(name = "deployment", help = "ID of the deployment to rollout")]
    pub deployment: Option<String>,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    let deployment_id = match options.deployment {
        Some(id) => id,

        None => {
            let project_id = state.ctx.current_project_error().id;

            let deployments = get_all_deployments(&state.http, &project_id).await?;
            ensure!(!deployments.is_empty(), "This project has no deployments");
            let deployments_fmt = format_deployments(&deployments, false);

            let idx = dialoguer::Select::new()
                .with_prompt("Select a deployment to delete")
                .items(&deployments_fmt)
                .default(0)
                .interact_opt()
                .expect("Failed to select deployment")
                .expect("No deployment selected");

            deployments[idx].id.clone()
        }
    };

    rollout(&state.http, &deployment_id).await?;

    log::info!("Rollling out new containers");

    Ok(())
}
