use anyhow::{ensure, Result};
use clap::Parser;

use crate::commands::containers::utils::create_containers;
use crate::commands::ignite::groups::utils::fetch_grouped_deployments;
use crate::state::State;

#[derive(Debug, Parser)]
#[clap(about = "Create containers for a deployment")]
#[group(skip)]
pub struct Options {
    #[clap(short, long, help = "ID of the deployment")]
    deployment: Option<String>,

    #[clap(help = "Number of containers to create")]
    count: Option<u64>,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    let deployment_id = match options.deployment {
        Some(id) => id,

        None => {
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

            deployments[idx].id.clone()
        }
    };

    let count = match options.count {
        Some(count) => count,
        None => dialoguer::Input::<u64>::new()
            .with_prompt("Number of containers to create")
            .interact()?,
    };

    ensure!(count > 0, "Count must be greater than 0");

    create_containers(&state.http, &deployment_id, count).await?;

    log::info!("Created {} containers", count);

    Ok(())
}
