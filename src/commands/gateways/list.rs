use anyhow::Result;
use clap::Parser;

use crate::commands::gateways::util::{format_gateways, get_all_gateways};
use crate::commands::ignite::util::{format_deployments, get_all_deployments};
use crate::state::State;

#[derive(Debug, Parser)]
#[clap(about = "List all gateways")]
pub struct Options {
    #[clap(name = "deployment", help = "NAME or ID of the deployment")]
    pub deployment: Option<String>,

    #[clap(
        short = 'q',
        long = "quiet",
        help = "Only print the IDs of the deployments"
    )]
    pub quiet: bool,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    let deployment_id = match options.deployment {
        Some(name) => {
            if name.starts_with("deployment_") {
                name
            } else {
                let project_id = state.ctx.current_project_error().id;

                let deployments = get_all_deployments(&state.http, &project_id).await?;

                deployments
                    .iter()
                    .find(|p| p.name == name || p.id == name)
                    .expect("Deployment not found")
                    .id
                    .clone()
            }
        }

        None => {
            let project_id = state.ctx.current_project_error().id;

            let deployments = get_all_deployments(&state.http, &project_id).await?;

            let deployments_fmt = format_deployments(&deployments, false);

            let idx = dialoguer::Select::new()
                .with_prompt("Select a deployment")
                .items(&deployments_fmt)
                .default(0)
                .interact_opt()
                .expect("Failed to select deployment")
                .expect("No deployment selected");

            let gateways = get_all_gateways(&state.http, &deployments[idx].id).await?;

            let gateways_fmt = format_gateways(&gateways, false);

            let idx = dialoguer::Select::new()
                .with_prompt("Select a gateway")
                .items(&gateways_fmt)
                .default(0)
                .interact_opt()
                .expect("Failed to select gateway")
                .expect("No gateway selected");

            gateways[idx].id.clone()
        }
    };

    let gateways = get_all_gateways(&state.http, &deployment_id).await?;

    if options.quiet {
        let ids = gateways
            .iter()
            .map(|d| d.id.as_str())
            .collect::<Vec<_>>()
            .join(" ");

        println!("{}", ids);
    } else {
        let containers_fmt = format_gateways(&gateways, true);

        println!("{}", containers_fmt.join("\n"));
    }

    Ok(())
}
