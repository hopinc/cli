use anyhow::{anyhow, ensure, Result};
use clap::Parser;

use super::ignite::builds::types::BuildState;
use super::ignite::builds::utils::get_all_builds;
use super::ignite::utils::{format_deployments, get_all_deployments, promote};
use crate::commands::projects::utils::format_project;
use crate::state::State;
use crate::store::hopfile::HopFile;

#[derive(Debug, Parser)]
#[clap(about = "Instantly roll back your deployment to a previous build")]
pub struct Options {
    #[clap(help = "ID of the deployment")]
    pub deployment: Option<String>,
}

pub async fn handle(options: &Options, state: State) -> Result<()> {
    let deployment_id = if let Some(ref id) = options.deployment {
        id.clone()
    } else if let Some(hopfile) = HopFile::find_current().await {
        hopfile.config.deployment_id
    } else {
        let project = state.ctx.current_project_error()?;

        log::info!("Using project: {}", format_project(&project));

        let deployments = get_all_deployments(&state.http, &project.id).await?;
        ensure!(!deployments.is_empty(), "No deployments found.");
        let deployments_fmt = format_deployments(&deployments, false);

        let idx = dialoguer::Select::new()
            .with_prompt("Select a deployment")
            .items(&deployments_fmt)
            .default(0)
            .interact()?;

        deployments[idx].id.clone()
    };

    let build_id = if let Some(build) = get_all_builds(&state.http, &deployment_id)
        .await?
        .into_iter()
        .find(|b| matches!(b.state, BuildState::Succeeded))
    {
        build.id
    } else {
        return Err(anyhow!("No successful builds found."));
    };

    promote(&state.http, &deployment_id, &build_id).await?;

    log::info!("Deployment `{deployment_id}` rolled back to build `{build_id}`");

    Ok(())
}
