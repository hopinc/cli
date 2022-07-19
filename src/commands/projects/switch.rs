use clap::Parser;

use crate::commands::projects::util::format_projects;
use crate::state::State;

#[derive(Debug, Parser)]
#[clap(name = "switch", about = "Switch to a different project")]
pub struct SwitchOptions {
    #[clap(name = "project", help = "Namespace or ID of the project to use")]
    pub project: Option<String>,
}

pub async fn handle_switch(options: SwitchOptions, mut state: State) -> Result<(), std::io::Error> {
    let projects = state
        .ctx
        .me
        .clone()
        .expect("You are not logged in. Please run `hop auth login` first.")
        .projects;

    if projects.is_empty() {
        panic!("No projects found");
    }

    let project = match options.project.or(state.ctx.clone().project_override) {
        Some(project) => state
            .ctx
            .clone()
            .find_project_by_id_or_namespace_error(project),
        None => {
            let projects_fmt = format_projects(&projects, &state.ctx.default_project);

            let idx = dialoguer::Select::new()
                .with_prompt("Select a project to set as default")
                .items(&projects_fmt)
                .default(if let Some(id) = state.ctx.default_project {
                    projects.iter().position(|p| p.id == id).unwrap_or(0)
                } else {
                    0
                })
                .interact_opt()
                .expect("Failed to select project")
                .expect("No project selected");

            projects[idx].clone()
        }
    };

    state.ctx.default_project = Some(project.id.clone());
    state.ctx.save().await?;

    log::info!(
        "Switched to project {} /{} ({})",
        project.name,
        project.namespace,
        project.id
    );

    Ok(())
}
