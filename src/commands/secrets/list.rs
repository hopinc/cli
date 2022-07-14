use structopt::StructOpt;

use crate::commands::secrets::types::Secrets;
use crate::state::State;

#[derive(Debug, StructOpt)]
#[structopt(about = "List all secrets")]
pub struct ListOptions {}

pub async fn handle_list(_options: ListOptions, state: State) -> Result<(), std::io::Error> {
    let project_id = state.ctx.current_project_error().id;

    let secrets = state
        .http
        .request::<Secrets>(
            "GET",
            format!("/projects/{}/secrets", project_id).as_str(),
            None,
        )
        .await
        .expect("Error while getting project info")
        .unwrap()
        .secrets;

    if secrets.is_empty() {
        panic!("No secrets found in this project");
    }

    let secrets_fmt = secrets
        .iter()
        .map(|s| format!(" {} ({})", s.name, s.id))
        .collect::<Vec<_>>();

    println!("Secrets:");
    println!("{}", secrets_fmt.join("\n"));

    Ok(())
}
