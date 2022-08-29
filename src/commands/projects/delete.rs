use anyhow::Result;
use clap::Parser;

use super::util::format_projects;
use crate::{commands::projects::util::format_project, state::State};

static CONFIRM_DELETE_PROJECT_MESSAGE: &str = "I am sure I want to delete the project named ";

#[derive(Debug, Parser)]
#[clap(about = "Delete a project")]
pub struct Options {
    #[clap(name = "project", help = "Namespace or ID of the project")]
    project: Option<String>,
    #[clap(short = 'f', long = "force", help = "Skip confirmation")]
    force: bool,
}

pub async fn handle(options: &Options, mut state: State) -> Result<()> {
    let projects = state.ctx.current.clone().unwrap().projects;

    let project = match options.project.clone() {
        Some(namespace) => projects
            .iter()
            .find(|p| p.namespace == namespace || p.id == namespace)
            .expect("Project not found"),

        None => {
            let projects_fmt = format_projects(&projects, false);

            let idx = dialoguer::Select::new()
                .with_prompt("Select a project")
                .items(&projects_fmt)
                .default(if let Some(current) = state.ctx.clone().current_project() {
                    projects
                        .iter()
                        .position(|p| p.id == current.id)
                        .unwrap_or(0)
                } else {
                    0
                })
                .interact_opt()
                .expect("Failed to select project")
                .expect("No project selected");

            &projects[idx]
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
            .expect("Failed to confirm deletion");

        assert!(
            output == CONFIRM_DELETE_PROJECT_MESSAGE.to_string() + &project.name,
            "Aborted deletion of `{}`",
            project.name
        );
    }

    state
        .http
        .request::<()>("DELETE", &format!("/projects/{}", project.id), None)
        .await?;

    if state.ctx.default_project == Some(project.id.to_string()) {
        state.ctx.default_project = None;
        state.ctx.save().await?;
    }

    log::info!("Project {} deleted", format_project(project));

    Ok(())
}
