use anyhow::{ensure, Result};
use clap::Parser;

use crate::commands::containers::utils::{format_containers, get_all_containers};
use crate::commands::ignite::utils::{format_deployments, get_all_deployments, get_deployment};
use crate::state::State;

#[derive(Debug, Parser)]
#[clap(about = "List all containers")]
pub struct Options {
    #[clap(name = "deployment", help = "ID of the deployment")]
    pub deployment: Option<String>,

    #[clap(
        short = 'q',
        long = "quiet",
        help = "Only print the IDs of the deployments"
    )]
    pub quiet: bool,
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

    let containers = get_all_containers(&state.http, &deployment.id).await?;

    if options.quiet {
        let ids = containers
            .iter()
            .map(|d| d.id.as_str())
            .collect::<Vec<_>>()
            .join(" ");

        println!("{}", ids);
    } else {
        let containers_fmt = format_containers(&containers, true);

        println!("{}", containers_fmt.join("\n"));
    }

    Ok(())
}
