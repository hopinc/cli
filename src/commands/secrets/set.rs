use crate::commands::secrets::util::{validate_name, SecretResponse};
use crate::state::State;
use crate::types::Base;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(about = "Set a secret")]
pub struct SetOptions {
    #[structopt(name = "name", help = "Name of the secret")]
    pub name: String,
    #[structopt(name = "value", help = "Value of the secret")]
    pub value: String,
}

pub async fn handle_set(options: SetOptions, state: State) -> Result<(), std::io::Error> {
    validate_name(&options.name).unwrap();

    let project_id = state.ctx.current_project().expect("Project not found").id;

    let secret = state
        .http
        .request::<Base<SecretResponse>>(
            "PUT",
            format!(
                "/projects/{}/secrets/{}",
                project_id,
                options.name.to_uppercase()
            )
            .as_str(),
            Some((options.value.into(), "text/plain")),
        )
        .await
        .expect("Error while setting secret")
        .unwrap()
        .data
        .secret;

    println!("Set secret: {} ({})", secret.name, secret.id);

    Ok(())
}
