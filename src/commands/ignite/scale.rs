use anyhow::Result;
use clap::Parser;

use super::utils::scale;
use crate::commands::ignite::groups::utils::fetch_grouped_deployments;
use crate::commands::ignite::utils::get_deployment;
use crate::state::State;

#[derive(Debug, Parser)]
#[clap(about = "Scale a deployment")]
#[group(skip)]
pub struct Options {
    #[clap(help = "ID of the deployment to scale")]
    pub deployment: Option<String>,

    #[clap(help = "Number of replicas to scale to")]
    pub scale: Option<u64>,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    let deployment = match options.deployment {
        Some(id) => get_deployment(&state.http, &id).await?,

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

            deployments[idx].clone()
        }
    };

    let scale_count = match options.scale {
        Some(scale) => scale,
        None => dialoguer::Input::<u64>::new()
            .with_prompt("Enter the number of containers to scale to")
            .default(deployment.container_count)
            .interact()?,
    };

    scale(&state.http, &deployment.id, scale_count).await?;

    log::info!("Scaling deployment to {} containers", scale_count);

    Ok(())
}
