use anyhow::{anyhow, Result};
use clap::Parser;

use crate::commands::secrets::types::SecretResponse;
use crate::commands::secrets::utils::validate_name;
use crate::state::State;

#[derive(Debug, Parser)]
#[clap(about = "Set a secret")]
#[group(skip)]
pub struct Options {
    #[clap(help = "Name of the secret")]
    name: String,
    #[clap(help = "Value of the secret")]
    value: String,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    validate_name(&options.name)?;

    let project_id = state.ctx.current_project_error()?.id;

    let secret = state
        .http
        .request::<SecretResponse>(
            "PUT",
            &format!(
                "/projects/{project_id}/secrets/{}",
                options.name.to_uppercase()
            ),
            Some((options.value.into(), "text/plain")),
        )
        .await?
        .ok_or_else(|| anyhow!("Error while parsing response"))?
        .secret;

    log::info!("Set secret: {} ({})", secret.name, secret.id);

    Ok(())
}
