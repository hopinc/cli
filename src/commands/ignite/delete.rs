use anyhow::{anyhow, Result};
use clap::Parser;

use crate::{
    commands::ignite::util::{delete_deployment, format_deployments, get_all_deployments},
    state::State,
};

#[derive(Debug, Parser)]
#[clap(about = "Delete a deployment")]
pub struct Options {
    #[clap(name = "deployment", help = "ID of the deployment to delete")]
    deployment: Option<String>,

    #[clap(short = 'f', long = "force", help = "Skip confirmation")]
    force: bool,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    let deployment_id = match options.deployment {
        Some(id) => id,

        None => {
            let project_id = state.ctx.current_project_error().id;

            let deployments = get_all_deployments(&state.http, &project_id).await?;
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

    if !options.force {
        dialoguer::Confirm::new()
            .with_prompt(format!(
                "Are you sure you want to delete `{}`?",
                deployment_id
            ))
            .interact_opt()?
            .ok_or_else(|| anyhow!("Aborted"))?;
    }

    delete_deployment(&state.http, &deployment_id).await?;

    log::info!("Deployment `{}` deleted", deployment_id);

    Ok(())
}
