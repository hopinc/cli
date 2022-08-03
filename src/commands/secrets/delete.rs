use clap::Parser;

use crate::commands::secrets::types::Secrets;
use crate::commands::secrets::util::validate_name;
use crate::state::State;

#[derive(Debug, Parser)]
#[clap(about = "Delete a secret")]
pub struct Options {
    #[structopt(name = "name", help = "Name of the secret")]
    pub name: Option<String>,
    #[structopt(long = "no-confirm", help = "Skip confirmation")]
    force: bool,
}

pub async fn handle(options: Options, state: State) -> Result<(), std::io::Error> {
    if let Some(ref name) = options.name {
        validate_name(name).unwrap();
    }

    let project_id = state.ctx.current_project().expect("Project not found").id;

    let secret_name = match options.name {
        Some(name) => name,
        None => {
            let secrests = state
                .http
                .request::<Secrets>(
                    "GET",
                    format!("/projects/{}/secrets", project_id).as_str(),
                    None,
                )
                .await
                .expect("Error while getting secrets")
                .unwrap()
                .secrets;

            assert!(!secrests.is_empty(), "No secrets found");

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

    if !options.force {
        let confirm = dialoguer::Confirm::new()
            .with_prompt(&format!(
                "Are you sure you want to delete secret {}?",
                secret_name
            ))
            .interact_opt()
            .expect("Failed to confirm");

        assert!(
            (confirm.is_some() || confirm.unwrap()),
            "Aborted deletion of `{}`",
            secret_name
        );
    }

    state
        .http
        .request::<()>(
            "DELETE",
            format!("/projects/{}/secrets/{}", project_id, secret_name).as_str(),
            None,
        )
        .await
        .expect("Error while deleting secret");

    log::info!("Secret `{}` deleted", secret_name);

    Ok(())
}
