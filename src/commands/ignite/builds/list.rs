use anyhow::{ensure, Result};
use clap::Parser;

use super::utils::{format_builds, get_all_builds};
use crate::commands::ignite::utils::{format_deployments, get_all_deployments};
use crate::state::State;

#[derive(Debug, Parser)]
#[clap(about = "List all builds in a deployment")]
pub struct Options {
    #[clap(help = "ID of the deployment")]
    pub deployment: Option<String>,

    #[clap(short, long, help = "Only print the IDs of the builds")]
    pub quiet: bool,
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
                .interact_opt()?
                .ok_or_else(|| anyhow::anyhow!("No deployment selected"))?;

            deployments[idx].id.clone()
        }
    };

    let builds = get_all_builds(&state.http, &deployment_id).await?;

    if options.quiet {
        let ids = builds
            .iter()
            .map(|d| d.id.as_str())
            .collect::<Vec<_>>()
            .join(" ");

        println!("{}", ids);
    } else {
        let builds_fmt = format_builds(&builds, true);

        println!("{}", builds_fmt.join("\n"));
    }

    Ok(())
}
