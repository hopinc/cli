use anyhow::{bail, ensure, Result};
use clap::Parser;

use super::utils::delete_container;
use crate::commands::containers::utils::{format_containers, get_all_containers};
use crate::commands::ignite::groups::utils::fetch_grouped_deployments;
use crate::state::State;

#[derive(Debug, Parser)]
#[clap(about = "Delete containers")]
#[group(skip)]
pub struct Options {
    #[clap(help = "IDs of the containers")]
    containers: Vec<String>,

    #[clap(short, long, help = "Skip confirmation")]
    force: bool,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    let containers = if !options.containers.is_empty() {
        options.containers
    } else {
        let (deployments_fmt, deployments, validator) =
            fetch_grouped_deployments(&state, false, true).await?;

        let idx = loop {
            let idx = dialoguer::Select::new()
                .with_prompt("Select a deployment")
                .items(&deployments_fmt)
                .default(0)
                .interact()?;

            if let Ok(idx) = validator(idx) {
                break idx;
            }

            console::Term::stderr().clear_last_lines(1)?
        };

        let containers = get_all_containers(&state.http, &deployments[idx].id).await?;
        ensure!(!containers.is_empty(), "No containers found");
        let containers_fmt = format_containers(&containers, false);

        let idxs = dialoguer::MultiSelect::new()
            .with_prompt("Select containers to delete")
            .items(&containers_fmt)
            .interact()?;

        containers
            .iter()
            .enumerate()
            .filter(|(i, _)| idxs.contains(i))
            .map(|(_, c)| c.id.clone())
            .collect()
    };

    if !options.force
        && !dialoguer::Confirm::new()
            .with_prompt(format!(
                "Are you sure you want to delete {} containers?",
                containers.len()
            ))
            .interact_opt()?
            .unwrap_or(false)
    {
        bail!("Aborted");
    }

    let mut delete_count = 0;

    for container in &containers {
        log::info!("Deleting container `{}`", container);

        if let Err(err) = delete_container(&state.http, container, false).await {
            log::error!("Failed to delete container `{}`: {}", container, err);
        } else {
            delete_count += 1;
        }
    }

    log::info!("Deleted {delete_count}/{} containers", containers.len());

    Ok(())
}
