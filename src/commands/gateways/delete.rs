use anyhow::{bail, Result};
use clap::Parser;

use crate::commands::gateways::util::{delete_gateway, format_gateways, get_all_gateways};
use crate::commands::ignite::groups::utils::fetch_grouped_deployments;
use crate::state::State;

#[derive(Debug, Parser)]
#[clap(about = "Delete gateways")]
#[group(skip)]
pub struct Options {
    #[clap(name = "gateways", help = "IDs of the gateways")]
    gateways: Vec<String>,

    #[clap(short = 'f', long = "force", help = "Skip confirmation")]
    force: bool,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    let gateways = if !options.gateways.is_empty() {
        options.gateways
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

        let gateways = get_all_gateways(&state.http, &deployments[idx].id).await?;
        let gateways_fmt = format_gateways(&gateways, false);

        let idxs = dialoguer::MultiSelect::new()
            .with_prompt("Select Gateways to delete")
            .items(&gateways_fmt)
            .interact()?;

        gateways
            .iter()
            .enumerate()
            .filter(|(i, _)| idxs.contains(i))
            .map(|(_, c)| c.id.clone())
            .collect()
    };

    if !options.force
        && !dialoguer::Confirm::new()
            .with_prompt(format!(
                "Are you sure you want to delete {} Gateways?",
                gateways.len()
            ))
            .interact_opt()?
            .unwrap_or(false)
    {
        bail!("Aborted");
    }

    let mut delete_count = 0;

    for gateway in &gateways {
        log::info!("Deleting Gateway `{gateway}`");

        if let Err(err) = delete_gateway(&state.http, gateway).await {
            log::error!("Failed to delete Gateway `{}`: {}", gateway, err);
        } else {
            delete_count += 1;
        }
    }

    log::info!("Deleted {delete_count}/{} Gateways", gateways.len());

    Ok(())
}
