use crate::state::State;
use crate::types::{Base, Projects};

pub async fn handle_switch(mut state: State) -> Result<(), std::io::Error> {
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

    let projects = user.data.projects;

    let projects_fmt = projects
        .iter()
        .map(|p| format!("{} ({})", p.name, p.namespace))
        .collect::<Vec<_>>();

    let idx = dialoguer::Select::new()
        .with_prompt("Select a project (use arrow keys and enter to select)")
        .items(&projects_fmt)
        .default(if let Some(id) = state.ctx.project {
            projects.iter().position(|p| p.id == id).unwrap()
        } else {
            0
        })
        .interact()
        .unwrap();

    state.ctx.project = Some(projects[idx].id.clone());
    state.ctx.save().await?;

    Ok(())
}
