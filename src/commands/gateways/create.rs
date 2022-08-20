use anyhow::{ensure, Result};
use clap::Parser;
use console::style;

use super::types::{GatewayProtocol, GatewayType};
use crate::commands::gateways::types::GatewayConfig;
use crate::commands::gateways::util::{create_gateway, update_gateway_config};
use crate::commands::ignite::util::{format_deployments, get_all_deployments};
use crate::state::State;

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
    #[clap(name = "deployment", help = "ID of the deployment")]
    pub deployment: Option<String>,

    #[clap(flatten)]
    pub config: GatewayOptions,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    let deployment_id = match options.deployment {
        Some(deployment) => deployment,

        None => {
            let project_id = state.ctx.current_project_error().id;

            let deployments = get_all_deployments(&state.http, &project_id).await?;
            ensure!(!deployments.is_empty(), "This project has no deployments");
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
        &GatewayConfig::default(),
    )?;

    let gateway = create_gateway(&state.http, &deployment_id, &gateway_config).await?;

    log::info!("Created gateway `{}`", gateway.id);

    if gateway.type_ == GatewayType::External {
        log::info!(
            "You can now access your app at {}",
            style(gateway.full_url()).underlined().bold()
        );
    }

    Ok(())
}
