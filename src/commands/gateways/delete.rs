use anyhow::{anyhow, ensure, Result};
use clap::Parser;

use crate::commands::{
    gateways::util::{delete_gateway, format_gateways, get_all_gateways},
    ignite::util::{format_deployments, get_all_deployments},
};
use crate::state::State;

#[derive(Debug, Parser)]
#[clap(about = "Delete gateways")]
pub struct Options {
    #[clap(name = "gateways", help = "IDs of the gateways", min_values = 0)]
    gateways: Vec<String>,

    #[clap(short = 'f', long = "force", help = "Skip confirmation")]
    force: bool,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    let gateways = if !options.gateways.is_empty() {
        options.gateways
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

        let deployment = deployments[idx].clone();

        let gateways = get_all_gateways(&state.http, &deployment.id).await?;

        let containers_fmt = format_gateways(&gateways, false);

        let idxs = dialoguer::MultiSelect::new()
            .with_prompt("Select gateways to delete")
            .items(&containers_fmt)
            .interact_opt()?
            .expect("No gateway selected");

        gateways
            .iter()
            .enumerate()
            .filter(|(i, _)| idxs.contains(i))
            .map(|(_, c)| c.id.clone())
            .collect()
    };

    if !options.force {
        dialoguer::Confirm::new()
            .with_prompt(format!(
                "Are you sure you want to delete {} gateways?",
                gateways.len()
            ))
            .interact_opt()?
            .ok_or_else(|| anyhow!("Aborted"))?;
    }

    let mut delete_count = 0;

    for gateway in &gateways {
        log::info!("Deleting gateway `{gateway}`");

        if let Err(err) = delete_gateway(&state.http, gateway).await {
            log::error!("Failed to delete gateway `{}`: {}", gateway, err);
        } else {
            delete_count += 1;
        }
    }

    log::info!("Deleted {delete_count}/{} gateways", gateways.len());

    Ok(())
}
