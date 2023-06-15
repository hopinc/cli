use anyhow::{ensure, Context, Result};
use clap::Parser;
use serde_json::Value;

use super::utils::format_projects;
use crate::commands::projects::utils::format_project;
use crate::state::State;
use crate::store::Store;

static CONFIRM_DELETE_PROJECT_MESSAGE: &str = "I am sure I want to delete the project named ";

#[derive(Debug, Parser)]
#[clap(about = "Delete a project")]
#[group(skip)]
pub struct Options {
    #[clap(help = "Namespace or ID of the project")]
    project: Option<String>,
    #[clap(short, long, help = "Skip confirmation")]
    force: bool,
}

pub async fn handle(options: Options, mut state: State) -> Result<()> {
    let projects = state.ctx.current.clone().unwrap().projects;

    let project = match options.project.clone() {
        Some(namespace) => state
            .ctx
            .find_project_by_id_or_namespace(&namespace)
            .with_context(|| format!("Project `{namespace}` not found"))?,

        None => {
            let projects_fmt = format_projects(&projects, false);

            let idx = dialoguer::Select::new()
                .with_prompt("Select a project")
                .items(&projects_fmt)
                .default(if let Some(current) = state.ctx.current_project() {
                    projects
                        .iter()
                        .position(|p| p.id == current.id)
                        .unwrap_or(0)
                } else {
                    0
                })
                .interact()?;

            projects[idx].clone()
        }
    };

    if !options.force {
        println!(
            "To confirm, input the following message `{}{}`",
            CONFIRM_DELETE_PROJECT_MESSAGE, project.name
        );

        let output = dialoguer::Input::<String>::new()
            .with_prompt("Message")
            .interact_text()
            .context("Failed to confirm deletion")?;

        ensure!(
            output == CONFIRM_DELETE_PROJECT_MESSAGE.to_string() + &project.name,
            "Aborted deletion of `{}`",
            project.name
        );
    }

    state
        .http
        .request::<Value>("DELETE", &format!("/projects/{}", project.id), None)
        .await?;

    if state.ctx.default_project == Some(project.id.to_string()) {
        state.ctx.default_project = None;
        state.ctx.save().await?;
    }

    log::info!("Project {} deleted", format_project(&project));

    Ok(())
}
