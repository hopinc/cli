use crate::state::State;
use crate::types::{Base, Projects};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "hop project ls", about = "🗒️ List all available projects")]
pub struct LsOptions {}

pub async fn handle_ls(state: State) -> Result<(), std::io::Error> {
    let response = state
        .http
        .client
        .get(format!("{}/users/@me", state.http.base_url))
        .send()
        .await
        .expect("Error while getting project info: {}");

    let user = response
        .json::<Base<Projects>>()
        .await
        .expect("Error while parsing json");

    let projects = user
        .data
        .projects
        .iter()
        .map(|p| format!("> {} ({}) ", p.name, p.namespace))
        .collect::<Vec<_>>();

    println!("Available projects:");
    println!("{}", projects.join("\n"));

    Ok(())
}
