use crate::commands::secrets::util::{validate_name, CreateParams, SecretResponse, UpdateParams};
use crate::state::State;
use crate::types::{Base, Secrets};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "set", about = "Set a secret")]
pub struct SetOptions {
    #[structopt(name = "name", help = "Name of the secret")]
    pub name: String,
    #[structopt(name = "value", help = "Value of the secret")]
    pub value: String,
}

pub async fn handle_set(options: SetOptions, state: State) -> Result<(), std::io::Error> {
    validate_name(&options.name).unwrap();

    let project_id = state.ctx.current_project().expect("Project not found").id;

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

    let idx = secrets
        .iter()
        // all secrets are upper case
        .position(|s| s.name == options.name.to_uppercase());

    // if the secret already exists, update it instead of creating a new one
    let (method, url, body) = match idx {
        Some(idx) => {
            let body = UpdateParams {
                value: options.value,
            };

            (
                "PUT",
                format!("/projects/{}/secrets/{}", project_id, &secrets[idx].id),
                serde_json::to_string(&body).unwrap(),
            )
        }
        None => {
            let secret = CreateParams {
                name: options.name.to_uppercase(),
                value: options.value,
            };
            (
                "POST",
                format!("/projects/{}/secrets", project_id,),
                serde_json::to_string(&secret).unwrap(),
            )
        }
    };

    let secret = state
        .http
        .request::<Base<SecretResponse>>(method, &url, Some(body))
        .await
        .expect("Error while setting secret")
        .unwrap()
        .data
        .secret;

    println!("Set secret: {} ({})", secret.name, secret.id);

    Ok(())
}
