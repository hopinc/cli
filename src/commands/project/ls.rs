use crate::state::State;
use crate::types::{Base, Projects};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "hop project ls", about = "🗒️ List all available projects")]
pub struct LsOptions {}

pub async fn handle_ls(state: State) -> Result<(), std::io::Error> {
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
        .map(|p| format!("> {} ({}) ", p.name, p.namespace))
        .collect::<Vec<_>>();

    println!("Available projects:");
    println!("{}", projects_fmt.join("\n"));

    Ok(())
}
