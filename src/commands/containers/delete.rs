use anyhow::{anyhow, ensure, Result};
use clap::Parser;

use super::utils::delete_container;
use crate::commands::containers::utils::{format_containers, get_all_containers};
use crate::commands::ignite::util::{format_deployments, get_all_deployments};
use crate::state::State;

#[derive(Debug, Parser)]
#[clap(about = "Delete containers")]
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
            .with_prompt("Select a deployment")
            .items(&deployments_fmt)
            .default(0)
            .interact_opt()
            .expect("Failed to select deployment")
            .expect("No deployment selected");

        let containers = get_all_containers(&state.http, &deployments[idx].id).await?;
        let containers_fmt = format_containers(&containers, false);

        let idxs = dialoguer::MultiSelect::new()
            .with_prompt("Select containers to delete")
            .items(&containers_fmt)
            .interact_opt()?
            .expect("No container selected");

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

    let mut delete_count = 0;

    for container in &containers {
        log::info!("Deleting container `{}`", container);

        if let Err(err) = delete_container(&state.http, container).await {
            log::error!("Failed to delete container `{}`: {}", container, err);
        } else {
            delete_count += 1;
        }
    }

    log::info!("Deleted {delete_count}/{} gateways", containers.len());

    Ok(())
}
