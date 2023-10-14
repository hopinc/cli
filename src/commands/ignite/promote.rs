use anyhow::{ensure, Result};
use clap::Parser;

use super::utils::promote;
use crate::commands::ignite::builds::types::BuildState;
use crate::commands::ignite::builds::utils::get_all_builds;
use crate::commands::ignite::groups::utils::fetch_grouped_deployments;
use crate::state::State;

#[derive(Debug, Parser)]
#[clap(about = "Rollback containers in a deployment")]
#[group(skip)]
pub struct Options {
    #[clap(help = "ID of the deployment")]
    pub deployment: Option<String>,

    #[clap(help = "ID of the build to rollback to")]
    pub build: Option<String>,
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

    let build_id = match options.build {
        Some(id) => id,

        None => {
            let builds = get_all_builds(&state.http, &deployment_id)
                .await?
                .into_iter()
                .filter(|b| matches!(b.state, BuildState::Succeeded))
                .collect::<Vec<_>>();

            ensure!(!builds.is_empty(), "No successful builds found");

            let idx = dialoguer::Select::new()
                .with_prompt("Select a build")
                .items(&builds.iter().map(|b| &b.id).collect::<Vec<_>>())
                .default(0)
                .interact()?;

            builds[idx].id.clone()
        }
    };

    promote(&state.http, &deployment_id, &build_id).await?;

    log::info!("Rolling out new containers");

    Ok(())
}
