use anyhow::{ensure, Result};
use clap::Parser;

use super::types::{GatewayProtocol, GatewayType};
use crate::commands::gateways::types::GatewayConfig;
use crate::commands::gateways::util::{create_gateway, update_gateway_config};
use crate::commands::ignite::utils::{format_deployments, get_all_deployments};
use crate::state::State;
use crate::utils::urlify;

#[derive(Debug, Parser, Default, PartialEq, Eq)]
pub struct GatewayOptions {
    #[clap(short = 'n', long = "name", help = "Name of the Gateway")]
    pub name: Option<String>,

    #[clap(short = 't', long = "type", help = "Type of the Gateway")]
    pub type_: Option<GatewayType>,

    #[clap(long = "protocol", help = "Protocol of the Gateway")]
    pub protocol: Option<GatewayProtocol>,

    #[clap(long = "target-port", help = "Port of the Gateway")]
    pub target_port: Option<u16>,

    #[clap(long = "internal-domain", help = "Internal domain of the Gateway")]
    pub internal_domain: Option<String>,
}

#[derive(Debug, Parser)]
#[clap(about = "Create a Gateway")]
#[group(skip)]
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
            let project_id = state.ctx.current_project_error()?.id;

            let deployments = get_all_deployments(&state.http, &project_id).await?;
            ensure!(!deployments.is_empty(), "No deployments found");
            let deployments_fmt = format_deployments(&deployments, false);

            let idx = dialoguer::Select::new()
                .with_prompt("Select a deployment")
                .items(&deployments_fmt)
                .default(0)
                .interact()?;

            deployments[idx].id.clone()
        }
    };

    let gateway_config = update_gateway_config(
        &options.config,
        options.config != GatewayOptions::default(),
        false,
        &GatewayConfig::default(),
    )?;

    let gateway = create_gateway(&state.http, &deployment_id, &gateway_config).await?;

    log::info!("Created Gateway `{}`", gateway.id);

    if gateway.type_ == GatewayType::External {
        log::info!(
            "You can now access your app at {}",
            urlify(&gateway.full_url())
        );
    }

    Ok(())
}
