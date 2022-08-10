use anyhow::ensure;
use clap::Parser;

use super::util::format_projects;
use crate::{commands::projects::util::format_project, state::State};

static CONFIRM_DELETE_PROJECT_MESSAGE: &str = "I am sure I want to delete the project named ";

#[derive(Debug, Parser)]
#[clap(about = "Delete a project")]
pub struct Options {
    #[clap(name = "namespace", help = "Namespace of the project")]
    namespace: Option<String>,
    #[clap(short = 'f', long = "force", help = "Skip confirmation")]
    force: bool,
}

pub async fn handle(options: &Options, mut state: State) -> anyhow::Result<()> {
    let projects = state.ctx.current.clone().unwrap().projects;

    ensure!(!projects.is_empty(), "No projects found");

    let project = match options.namespace.clone() {
        Some(namespace) => {
            let project = projects
                .iter()
                .find(|p| p.namespace == namespace)
                .expect("Project not found");

            project.clone()
        }
        None => {
            let projects_fmt = format_projects(&projects, false);

            let idx = dialoguer::Select::new()
                .with_prompt("Select a project to delete")
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

    log::info!("Project {} deleted", format_project(&project));

    Ok(())
}
