use anyhow::Result;
use clap::Parser;

use super::utils::rollout;
use crate::{commands::ignite::groups::utils::fetch_grouped_deployments, state::State};

#[derive(Debug, Parser)]
#[clap(about = "Rollout new containers to a deployment")]
#[group(skip)]
pub struct Options {
    #[clap(help = "ID of the deployment")]
    pub deployment: Option<String>,
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

    rollout(&state.http, &deployment_id).await?;

    log::info!("Rolling out new containers");

    Ok(())
}
