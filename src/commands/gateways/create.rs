use anyhow::Result;
use clap::Parser;

use crate::commands::gateways::types::Gateway;
use crate::commands::gateways::util::{create_gateway, update_gateway_config};
use crate::commands::ignite::util::{format_deployments, get_all_deployments};
use crate::state::State;

use super::types::{GatewayProtocol, GatewayType};

#[derive(Debug, Parser, Default, PartialEq)]
pub struct GatewayOptions {
    #[clap(short = 'n', long = "name", help = "Name of the gateway")]
    pub name: Option<String>,

    #[clap(short = 't', long = "type", help = "Type of the gateway")]
    pub type_: Option<GatewayType>,

    #[clap(long = "protocol", help = "Protocol of the gateway")]
    pub protocol: Option<GatewayProtocol>,

    #[clap(long = "target-port", help = "Port of the gateway")]
    pub target_port: Option<u16>,

    #[clap(long = "internal-domain", help = "Internal domain of the gateway")]
    pub internal_domain: Option<String>,
}

#[derive(Debug, Parser)]
#[clap(about = "Create a gateway")]
pub struct Options {
    #[clap(name = "deployment", help = "NAME or ID of the deployment")]
    pub deployment: Option<String>,

    #[clap(flatten)]
    pub config: GatewayOptions,
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

            deployments[idx].id.clone()
        }
    };

    let gateway_config = update_gateway_config(
        &options.config,
        options.config != GatewayOptions::default(),
        &Gateway::default(),
    )?;

    let gateway = create_gateway(&state.http, &deployment_id, &gateway_config).await?;

    log::info!("Created gateway `{}`", gateway.id);

    Ok(())
}
