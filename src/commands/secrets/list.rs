use crate::state::State;
use crate::types::{Base, Secrets};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "hop secrets list", about = "🗒️ List all secrets names")]
pub struct ListOptions {}

pub async fn handle_list(state: State) -> Result<(), std::io::Error> {
    let project_id = state.ctx.current_project().unwrap();

    let secrets = state
        .http
        .request::<Base<Secrets>>(
            "GET",
            format!("/projects/{}/secrets", project_id).as_str(),
            None,
        )
        .await
        .expect("Error while getting project info")
        .unwrap()
        .data
        .secrets;

    if secrets.len() == 0 {
        panic!("No secrets found in this project");
    }

    let secrets_fmt = secrets
        .iter()
        .map(|s| format!(" {} ({})", s.name, s.id))
        .collect::<Vec<_>>();

    println!("Available secrets:");
    println!("{}", secrets_fmt.join("\n"));

    Ok(())
}
