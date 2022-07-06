use crate::commands::secrets::util::validate_name;
use crate::state::State;
use crate::types::{Base, Secrets};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(about = "Delete a secret")]
pub struct DeleteOptions {
    #[structopt(name = "name", help = "Name of the secret")]
    pub name: Option<String>,
}

pub async fn handle_delete(options: DeleteOptions, state: State) -> Result<(), std::io::Error> {
    if let Some(ref name) = options.name {
        validate_name(&name).unwrap();
    }

    let project_id = state.ctx.current_project().expect("Project not found").id;

    let secret_name = match options.name {
        Some(name) => name,
        None => {
            let secrests = state
                .http
                .request::<Base<Secrets>>(
                    "GET",
                    format!("/projects/{}/secrets", project_id).as_str(),
                    None,
                )
                .await
                .expect("Error while getting secrets")
                .unwrap()
                .data
                .secrets;

            if secrests.is_empty() {
                panic!("No secrets found");
            }

            let secrets_fmt = secrests
                .iter()
                .map(|s| format!(" {} ({})", s.name, s.id))
                .collect::<Vec<_>>();

            let idx = dialoguer::Select::new()
                .with_prompt("Select a secret to delete")
                .items(&secrets_fmt)
                .default(0)
                .interact_opt()
                .expect("Failed to select secret")
                .expect("No secret selected");

            secrests[idx].name.clone()
        }
    };

    state
        .http
        .request::<()>(
            "DELETE",
            format!("/projects/{}/secrets/{}", project_id, secret_name).as_str(),
            None,
        )
        .await
        .expect("Error while deleting secret");

    println!("Secret `{}` deleted", secret_name);

    Ok(())
}
