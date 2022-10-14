use anyhow::{ensure, Result};
use clap::Parser;

use crate::commands::ignite::utils::{format_deployments, get_all_deployments};
use crate::state::State;

#[derive(Debug, Parser)]
#[clap(about = "Create Health Checks for a deployment")]
pub struct Options {
    #[clap(short = 'c', long = "health-check", help = "ID of the Health Check")]
    pub health_check: Option<String>,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    let health_check = match options.health_check {
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

    Ok(())
}
