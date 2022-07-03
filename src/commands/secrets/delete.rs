use crate::commands::secrets::util::validate_name;
use crate::state::State;
use crate::types::{Base, Secrets};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "hop secrets delete", about = "🗒️ Delete a secret")]
pub struct DeleteOptions {
    #[structopt(name = "name", help = "Name of the secret")]
    pub name: Option<String>,
}

pub async fn handle_delete(options: DeleteOptions, state: State) -> Result<(), std::io::Error> {
    if let Some(ref name) = options.name {
        validate_name(&name).unwrap();
    }

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

    let idx = match options.name {
        Some(name) => {
            let idx = secrets
                .iter()
                // all secrets are upper case
                .position(|s| s.name == name.to_uppercase());
            match idx {
                Some(idx) => idx,
                None => panic!("No secret found"),
            }
        }
        None => dialoguer::Select::new()
            .with_prompt("Select a secret to delete")
            .items(&secrets_fmt)
            .default(0)
            .interact_opt()
            .expect("Failed to select secret")
            .expect("No secret selected"),
    };

    let secret = &secrets[idx];

    state
        .http
        .request::<()>(
            "DELETE",
            format!("/projects/{}/secrets/{}", project_id, secret.id).as_str(),
            None,
        )
        .await
        .expect("Error while deleting secret");

    println!("Secret `{}` ({}) deleted", secret.name, secret.id);

    Ok(())
}
