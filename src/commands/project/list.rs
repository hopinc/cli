use crate::state::State;
use crate::types::{Base, Projects};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "hop project list", about = "🗒️ List all available projects")]
pub struct ListOptions {}

pub async fn handle_list(state: State) -> Result<(), std::io::Error> {
    let projects = state
        .http
        .request::<Base<Projects>>("GET", "/users/@me", None)
        .await
        .expect("Error while getting project info")
        .unwrap()
        .data
        .projects;

    if projects.len() == 0 {
        panic!("No projects found");
    }

    let projects_fmt = projects
        .iter()
        .map(|p| format!(" {} @{} ({})", p.name, p.namespace, p.id))
        .collect::<Vec<_>>();

    println!("Available projects:");
    println!("{}", projects_fmt.join("\n"));

    Ok(())
}
