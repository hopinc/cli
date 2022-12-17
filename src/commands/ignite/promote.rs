use anyhow::{ensure, Result};
use clap::Parser;

use super::utils::{format_deployments, get_all_deployments, promote};
use crate::commands::ignite::builds::types::BuildState;
use crate::commands::ignite::builds::utils::get_all_builds;
use crate::state::State;

#[derive(Debug, Parser)]
#[clap(about = "Rollback containers in a deployment")]
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
            let project_id = state.ctx.current_project_error().id;

            let deployments = get_all_deployments(&state.http, &project_id).await?;
            ensure!(!deployments.is_empty(), "No deployments found");
            let deployments_fmt = format_deployments(&deployments, false);

            let idx = dialoguer::Select::new()
                .with_prompt("Select a deployment")
                .items(&deployments_fmt)
                .default(0)
                .interact_opt()?
                .ok_or_else(|| anyhow::anyhow!("No build selected"))?;

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
                .interact_opt()?
                .ok_or_else(|| anyhow::anyhow!("No build selected"))?;

            builds[idx].id.clone()
        }
    };

    promote(&state.http, &deployment_id, &build_id).await?;

    log::info!("Rolling out new containers");

    Ok(())
}
