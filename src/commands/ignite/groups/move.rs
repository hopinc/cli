use anyhow::{ensure, Result};
use clap::Parser;
use console::style;

use crate::commands::ignite::groups::utils::{fetch_grouped_deployments, format_groups};
use crate::commands::ignite::utils::get_deployment;
use crate::config::EXEC_NAME;
use crate::state::State;

#[derive(Debug, Parser)]
#[clap(about = "Move a Deployment to a group")]
#[group(skip)]
pub struct Options {
    #[clap(help = "The ID of the group, or \"none\" to remove from a group")]
    pub group: Option<String>,
    #[clap(help = "Deployment ID to add")]
    pub deployment: Option<String>,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    let project = state.ctx.current_project_error()?;

    let deployment = if let Some(deployment) = options.deployment {
        get_deployment(&state.http, &deployment).await?
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

        deployments[idx].to_owned()
    };

    let group = if let Some(group) = options.group {
        if group.is_empty() || ["none", "null"].contains(&group.to_lowercase().as_str()) {
            None
        } else {
            Some(group)
        }
    } else {
        let mut groups = state.hop.ignite.groups.get_all(&project.id).await?;

        ensure!(
            !groups.is_empty(),
            "No groups found, create one with `{EXEC_NAME} ignite groups create`"
        );

        groups.sort_unstable_by_key(|group| group.position);

        let mut formated = format_groups(&groups)?;

        if deployment.group_id.is_some() {
            formated.push(style("None (remove from a group)").white().to_string());
        }

        let dialoguer_groups = dialoguer::Select::new()
            .with_prompt("Select group")
            .default(0)
            .items(&formated)
            .interact()?;

        if dialoguer_groups == groups.len() - 1 {
            None
        } else {
            Some(groups[dialoguer_groups].id.clone())
        }
    };

    state
        .hop
        .ignite
        .groups
        .move_deployment(group.as_deref(), &deployment.id)
        .await?;

    log::info!("Added deployment to group");

    Ok(())
}
