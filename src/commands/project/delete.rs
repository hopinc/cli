use crate::state::State;
use crate::types::{Base, Projects};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "hop project delete", about = "⚠️ Delete a project")]
pub struct DeleteOptions {}

pub async fn handle_delete(
    _options: DeleteOptions,
    mut state: State,
) -> Result<(), std::io::Error> {
    let projects = state
        .http
        .request::<Base<Projects>>("GET", "/users/@me", None)
        .await
        .expect("Error while getting project info")
        .unwrap()
        .data
        .projects;

    let projects_fmt = projects
        .iter()
        .map(|p| format!("{} ({})", p.name, p.namespace))
        .collect::<Vec<_>>();

    let idx = dialoguer::Select::new()
        .with_prompt("Select a project to delete (use arrow keys and enter to select)")
        .items(&projects_fmt)
        .default(if let Some(id) = state.ctx.project.clone() {
            projects.iter().position(|p| p.id == id).unwrap_or(0)
        } else {
            0
        })
        .interact_opt()
        .unwrap();

    let project = &projects[idx.unwrap_or_else(|| {
        eprintln!("Project not found");
        std::process::exit(1)
    })];

    state
        .http
        .request::<()>("DELETE", format!("/projects/{}", project.id).as_str(), None)
        .await
        .expect("Error while deleting project");

    println!(
        "Project \"{}\" ({}) deleted",
        project.name, project.namespace
    );

    if state.ctx.project == Some(project.id.to_string()) {
        state.ctx.project = None;
        state.ctx.save().await?;
    }

    Ok(())
}
