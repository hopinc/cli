use anyhow::{anyhow, ensure, Result};
use clap::Parser;

use super::utils::delete_container;
use crate::commands::{
    containers::utils::{format_containers, get_all_containers},
    ignite::util::{format_deployments, get_all_deployments},
};
use crate::state::State;

#[derive(Debug, Parser)]
#[clap(about = "Delete container(s)")]
pub struct Options {
    #[clap(name = "containers", help = "IDs of the containers", min_values = 0)]
    containers: Vec<String>,

    #[clap(short = 'f', long = "force", help = "Skip confirmation")]
    force: bool,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    let containers = if !options.containers.is_empty() {
        options.containers
    } else {
        let project_id = state.ctx.current_project_error().id;

        let deployments = get_all_deployments(&state.http, &project_id).await?;

        ensure!(!deployments.is_empty(), "No deployments found");

        let deployments_fmt = format_deployments(&deployments, false);

        let idx = dialoguer::Select::new()
            .with_prompt("Select a deployment to list containers of")
            .items(&deployments_fmt)
            .default(0)
            .interact_opt()
            .expect("Failed to select deployment")
            .expect("No deployment selected");

        let deployment = deployments[idx].clone();

        let containers = get_all_containers(&state.http, &deployment.id).await?;

        let containers_fmt = format_containers(&containers, false);

        let idxs = dialoguer::MultiSelect::new()
            .with_prompt("Select containers to delete")
            .items(&containers_fmt)
            .interact_opt()?
            .expect("No containers selected");

        containers
            .iter()
            .enumerate()
            .filter(|(i, _)| idxs.contains(i))
            .map(|(_, c)| c.id.clone())
            .collect()
    };

    if !options.force {
        dialoguer::Confirm::new()
            .with_prompt(format!(
                "Are you sure you want to delete {} containers?",
                containers.len()
            ))
            .interact_opt()?
            .ok_or_else(|| anyhow!("Aborted"))?;
    }

    for container in containers {
        log::info!("Deleting container `{}`", container);

        delete_container(&state.http, &container).await?;
    }

    Ok(())
}
