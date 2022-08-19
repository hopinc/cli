use anyhow::{ensure, Result};
use clap::Parser;

use super::create::GatewayOptions;
use crate::commands::gateways::types::GatewayConfig;
use crate::commands::gateways::util::{
    format_gateways, get_all_gateways, get_gateway, update_gateway, update_gateway_config,
};
use crate::commands::ignite::util::{format_deployments, get_all_deployments};
use crate::state::State;

#[derive(Debug, Parser)]
#[clap(about = "Update a gateway")]
pub struct Options {
    #[clap(name = "gateway", help = "ID of the gateway")]
    pub gateway: Option<String>,

    #[clap(flatten)]
    pub config: GatewayOptions,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    let gateway = match options.gateway {
        Some(gateway_id) => get_gateway(&state.http, &gateway_id).await?,

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

            let gateways = get_all_gateways(&state.http, &deployments[idx].id).await?;
            let gateways_fmt = format_gateways(&gateways, false);

            let idx = dialoguer::Select::new()
                .with_prompt("Select a gateway to update")
                .default(0)
                .items(&gateways_fmt)
                .interact_opt()?
                .expect("No gateways selected");

            gateways[idx].clone()
        }
    };

    let gateway_config = update_gateway_config(
        &options.config,
        options.config != GatewayOptions::default(),
        &GatewayConfig::from_gateway(&gateway),
    )?;

    update_gateway(&state.http, &gateway.id, &gateway_config).await?;

    log::info!("Updated gateway `{}`", gateway.id);

    Ok(())
}
