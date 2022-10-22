use anyhow::{bail, ensure, Result};
use clap::Parser;

use crate::commands::ignite::utils::{delete_deployment, format_deployments, get_all_deployments};
use crate::state::State;

#[derive(Debug, Parser)]
#[clap(about = "Delete a deployment")]
pub struct Options {
    #[clap(help = "ID of the deployment to delete")]
    deployment: Option<String>,

    #[clap(short, long, help = "Skip confirmation")]
    force: bool,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    let deployment_id = match options.deployment {
        Some(id) => id,

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

            deployments[idx].id.clone()
        }
    };

    if !options.force
        && !dialoguer::Confirm::new()
            .with_prompt("Are you sure you want to delete the deployment?")
            .interact_opt()?
            .unwrap_or(false)
    {
        bail!("Aborted");
    }

    delete_deployment(&state.http, &deployment_id).await?;

    log::info!("Deployment `{}` deleted", deployment_id);

    Ok(())
}
