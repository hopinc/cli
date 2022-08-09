use anyhow::Result;
use clap::Parser;
use std::io::Write;

use crate::commands::ignite::types::MultipleDeployments;
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

    let deployments = state
        .http
        .request::<MultipleDeployments>(
            "GET",
            &format!("/ignite/deployments?project={}", project_id),
            None,
        )
        .await
        .expect("Error while getting deployments")
        .unwrap()
        .deployments;

    assert!(!deployments.is_empty(), "No deployments found");

    let deployment = match options.deployment {
        Some(name) => {
            let deployment = deployments
                .iter()
                .find(|p| p.name == name || p.id == name)
                .expect("Deployment not found");
            deployment.clone()
        }
        None => {
            let deployments_fmt = deployments
                .iter()
                .map(|d| format!("{} ({})", d.name, d.id))
                .collect::<Vec<_>>();

            let idx = dialoguer::Select::new()
                .with_prompt("Select a deployment to delete")
                .items(&deployments_fmt)
                .default(0)
                .interact_opt()
                .expect("Failed to select deployment")
                .expect("No deployment selected");

            deployments[idx].clone()
        }
    };

    let mut buff = vec![];

    for (key, value) in deployment.config.env {
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
