use anyhow::{ensure, Result};
use clap::Parser;

use super::utils::{format_health_state, get_health_state};
use crate::commands::ignite::utils::{format_deployments, get_all_deployments};
use crate::state::State;

#[derive(Debug, Parser)]
#[clap(about = "Create Health Checks for a deployment")]
#[group(skip)]
pub struct Options {
    #[clap(help = "ID of the Deployment")]
    pub deployment: Option<String>,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    let deployment_id = match options.deployment {
        Some(id) => id,

        None => {
            let project_id = state.ctx.current_project_error()?.id;

            let deployments = get_all_deployments(&state.http, &project_id).await?;
            ensure!(!deployments.is_empty(), "No deployments found");
            let deployments_fmt = format_deployments(&deployments, false);

            let idx = dialoguer::Select::new()
                .with_prompt("Select a deployment")
                .items(&deployments_fmt)
                .default(0)
                .interact()?;

            deployments[idx].id.clone()
        }
    };

    let health_state = get_health_state(&state.http, &deployment_id).await?;
    let health_state_fmt = format_health_state(&health_state, true);

    println!("{}", health_state_fmt.join("\n"));

    Ok(())
}
