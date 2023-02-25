use anyhow::{ensure, Result};
use clap::Parser;

use super::utils::{format_health_checks, get_all_health_checks};
use crate::commands::ignite::utils::{format_deployments, get_all_deployments};
use crate::state::State;

#[derive(Debug, Parser)]
#[clap(about = "List Health Checks in a deployment")]
pub struct Options {
    #[clap(help = "ID of the deployment")]
    pub deployment: Option<String>,

    #[clap(short, long, help = "Only print the IDs of the Health Checks")]
    pub quiet: bool,
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

    let health_checks = get_all_health_checks(&state.http, &deployment_id).await?;

    if options.quiet {
        let ids = health_checks
            .iter()
            .map(|d| d.id.as_str())
            .collect::<Vec<_>>()
            .join(" ");

        println!("{ids}");
    } else {
        let health_checks_fmt = format_health_checks(&health_checks, true);

        println!("{}", health_checks_fmt.join("\n"));
    }

    Ok(())
}
