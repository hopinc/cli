use crate::state::State;
use crate::types::{Base, Projects};

pub async fn handle_ls(state: State) -> Result<(), std::io::Error> {
    let request = state
        .client
        .http
        .get(format!("{}/users/@me", state.client.base_url))
        .send()
        .await;

    if request.is_err() {
        eprintln!("Error while getting project info: {}", request.unwrap_err());
        std::process::exit(1);
    }

    let response = request.unwrap();

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
