use anyhow::{ensure, Result};
use clap::Parser;

use crate::commands::ignite::groups::utils::{fetch_grouped_deployments, format_groups};
use crate::config::EXEC_NAME;
use crate::state::State;

#[derive(Debug, Parser)]
#[clap(about = "Add a deployment to an Ignite group")]
#[group(skip)]
pub struct Options {
    #[clap(help = "The ID of the group")]
    pub group: Option<String>,
    #[clap(help = "Deployment ID to add")]
    pub deployment: Option<String>,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    let project = state.ctx.current_project_error()?;

    let group = if let Some(group) = options.group {
        group
    } else {
        let mut groups = state.hop.ignite.groups.get_all(&project.id).await?;

        ensure!(
            !groups.is_empty(),
            "No groups found, create one with `{EXEC_NAME} ignite groups create`"
        );

        groups.sort_unstable_by_key(|group| group.position);

        let dialoguer_groups = dialoguer::Select::new()
            .with_prompt("Select group")
            .default(0)
            .items(&format_groups(&groups)?)
            .interact()?;

        groups[dialoguer_groups].id.clone()
    };

    let deployments = if let Some(deployment) = options.deployment {
        deployment
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

        deployments[idx].id.clone()
    };

    state
        .hop
        .ignite
        .groups
        .add_deployment(&group, &deployments)
        .await?;

    log::info!("Added deployment to group");

    Ok(())
}
