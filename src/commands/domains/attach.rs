use anyhow::{ensure, Result};
use clap::Parser;

use super::util::attach_domain;
use crate::commands::gateways::util::{format_gateways, get_all_gateways};
use crate::commands::ignite::utils::{format_deployments, get_all_deployments};
use crate::state::State;

#[derive(Debug, Parser)]
#[clap(about = "Attach a domain to a Gateway")]
pub struct Options {
    #[clap(help = "ID of the Gateway")]
    pub gateway: Option<String>,

    #[clap(help = "Name of the domain")]
    pub domain: Option<String>,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    let gateway_id = match options.gateway {
        Some(id) => id,

        None => {
            let project_id = state.ctx.current_project_error().id;

            let deployments = get_all_deployments(&state.http, &project_id).await?;
            ensure!(!deployments.is_empty(), "No deployments found");
            let deployments_fmt = format_deployments(&deployments, false);

            let idx = dialoguer::Select::new()
                .with_prompt("Select a deployment")
                .items(&deployments_fmt)
                .default(0)
                .interact_opt()
                .expect("Failed to select deployment")
                .expect("No deployment selected");

            let gateways = get_all_gateways(&state.http, &deployments[idx].id).await?;
            ensure!(!gateways.is_empty(), "No Gateways found");
            let gateways_fmt = format_gateways(&gateways, false);

            let idx = dialoguer::Select::new()
                .with_prompt("Select a Gateway")
                .default(0)
                .items(&gateways_fmt)
                .interact_opt()?
                .expect("No Gateways selected");

            gateways[idx].id.clone()
        }
    };

    let domain = match options.domain {
        Some(name) => name,

        None => dialoguer::Input::<String>::new()
            .with_prompt("Enter the domain name")
            .interact()?,
    };

    attach_domain(&state.http, &gateway_id, &domain).await?;

    log::info!("Attached domain `{}` to Gateway `{}`", domain, gateway_id);
    log::info!("Please create a non-proxied DNS record pointing to the following");
    println!("\tCNAME {domain} -> border.hop.io");

    Ok(())
}
