use anyhow::Result;
use clap::Parser;
use std::io::Write;

use crate::commands::ignite::util::{format_deployments, get_all_deployments};
use crate::commands::secrets::util::get_secret_name;
use crate::state::State;

#[derive(Debug, Parser)]
#[clap(about = "Get current deployments env values")]
pub struct Options {
    #[clap(
        name = "deployment",
        help = "NAME or ID of the deployment to get env values"
    )]
    pub deployment: Option<String>,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    let project_id = state.ctx.current_project_error().id;

    let deployments = get_all_deployments(&state.http, &project_id).await?;

    let deployment = match options.deployment {
        Some(deployment) => deployments
            .iter()
            .find(|d| d.name == deployment || d.id == deployment)
            .ok_or_else(|| anyhow::anyhow!("Deployment not found"))?,
        None => {
            let deployments_fmt = format_deployments(&deployments, false);

            let idx = dialoguer::Select::new()
                .with_prompt("Select a deployment to delete")
                .items(&deployments_fmt)
                .default(0)
                .interact_opt()
                .expect("Failed to select deployment")
                .expect("No deployment selected");

            &deployments[idx]
        }
    };

    let mut buff = vec![];

    for (key, value) in deployment.config.env.clone() {
        let value = if let Some(secret_name) = get_secret_name(&value) {
            format!("{{{secret_name}}}")
        } else {
            value
        };

        writeln!(buff, "{key}={value}")?;
    }

    print!("{}", String::from_utf8(buff)?);

    Ok(())
}
