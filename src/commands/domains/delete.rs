use anyhow::{ensure, Result};
use clap::Parser;

use super::util::{delete_domain, format_domains, get_all_domains};
use crate::commands::gateways::util::{format_gateways, get_all_gateways};
use crate::commands::ignite::utils::{format_deployments, get_all_deployments};
use crate::state::State;

#[derive(Debug, Parser)]
#[clap(about = "Detach a domain from a Gateway")]
pub struct Options {
    #[clap(help = "ID of the domain")]
    pub domain: Option<String>,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    let domain_id = match options.domain {
        Some(id) => id,

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

            let gateways = get_all_gateways(&state.http, &deployments[idx].id).await?;
            ensure!(!gateways.is_empty(), "No Gateways found");
            let gateways_fmt = format_gateways(&gateways, false);

            let idx = dialoguer::Select::new()
                .with_prompt("Select a Gateway")
                .default(0)
                .items(&gateways_fmt)
                .interact()?;

            let domains = get_all_domains(&state.http, &gateways[idx].id).await?;
            let domains_fmt = format_domains(&domains, false);

            let idx = dialoguer::Select::new()
                .with_prompt("Select a domain")
                .default(0)
                .items(&domains_fmt)
                .interact()?;

            domains[idx].id.clone()
        }
    };

    delete_domain(&state.http, &domain_id).await?;

    log::info!("Domain `{domain_id}` detached");

    Ok(())
}
