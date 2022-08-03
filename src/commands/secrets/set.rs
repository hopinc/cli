use clap::Parser;

use crate::commands::secrets::types::SecretResponse;
use crate::commands::secrets::util::validate_name;
use crate::state::State;

#[derive(Debug, Parser)]
#[structopt(about = "Set a secret")]
pub struct Options {
    #[structopt(name = "name", help = "Name of the secret")]
    pub name: String,
    #[structopt(name = "value", help = "Value of the secret")]
    pub value: String,
}

pub async fn handle(options: Options, state: State) -> Result<(), std::io::Error> {
    validate_name(&options.name)?;

    let project_id = state.ctx.current_project().expect("Project not found").id;

    let secret = state
        .http
        .request::<SecretResponse>(
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
        .secret;

    log::info!("Set secret: {} ({})", secret.name, secret.id);

    Ok(())
}
