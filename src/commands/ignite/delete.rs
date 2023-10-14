use anyhow::{bail, Result};
use clap::Parser;

use crate::{
    commands::ignite::{groups::utils::fetch_grouped_deployments, utils::delete_deployment},
    state::State,
};

#[derive(Debug, Parser)]
#[clap(about = "Delete a deployment")]
#[group(skip)]
pub struct Options {
    #[clap(help = "ID of the deployment to delete")]
    deployment: Option<String>,

    #[clap(short, long, help = "Skip confirmation")]
    force: bool,
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

    if !options.force
        && !dialoguer::Confirm::new()
            .with_prompt("Are you sure you want to delete the deployment?")
            .interact_opt()?
            .unwrap_or(false)
    {
        bail!("Aborted");
    }

    delete_deployment(&state.http, &deployment_id).await?;

    log::info!("Deployment `{}` deleted", deployment_id);

    Ok(())
}
