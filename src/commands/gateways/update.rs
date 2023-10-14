use anyhow::Result;
use clap::Parser;

use super::create::GatewayOptions;
use crate::commands::gateways::types::GatewayConfig;
use crate::commands::gateways::util::{
    format_gateways, get_all_gateways, get_gateway, update_gateway, update_gateway_config,
};
use crate::commands::ignite::groups::utils::fetch_grouped_deployments;
use crate::state::State;

#[derive(Debug, Parser)]
#[clap(about = "Update a Gateway")]
#[group(skip)]
pub struct Options {
    #[clap(name = "gateway", help = "ID of the Gateway")]
    pub gateway: Option<String>,

    #[clap(flatten)]
    pub config: GatewayOptions,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    let gateway = match options.gateway {
        Some(gateway_id) => get_gateway(&state.http, &gateway_id).await?,

        None => {
            let (deployments_fmt, deployments, validator) =
                fetch_grouped_deployments(&state, false, true).await?;

            let idx = loop {
                let idx = dialoguer::Select::new()
                    .with_prompt("Select a deployment")
                    .items(&deployments_fmt)
                    .default(0)
                    .interact()?;

                if let Ok(idx) = validator(idx) {
                    break idx;
                }

                console::Term::stderr().clear_last_lines(1)?
            };

            let gateways = get_all_gateways(&state.http, &deployments[idx].id).await?;
            let gateways_fmt = format_gateways(&gateways, false);

            let idx = dialoguer::Select::new()
                .with_prompt("Select a Gateway to update")
                .default(0)
                .items(&gateways_fmt)
                .interact()?;

            gateways[idx].clone()
        }
    };

    let gateway_config = update_gateway_config(
        &options.config,
        options.config != GatewayOptions::default(),
        true,
        &GatewayConfig::from_gateway(&gateway),
    )?;

    update_gateway(&state.http, &gateway.id, &gateway_config).await?;

    log::info!("Updated Gateway `{}`", gateway.id);

    Ok(())
}
