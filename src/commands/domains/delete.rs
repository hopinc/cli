use anyhow::{ensure, Result};
use clap::Parser;

use super::util::{delete_domain, format_domains, get_all_domains};
use crate::commands::gateways::util::{format_gateways, get_all_gateways};
use crate::commands::ignite::groups::utils::fetch_grouped_deployments;
use crate::state::State;

#[derive(Debug, Parser)]
#[clap(about = "Detach a domain from a Gateway")]
#[group(skip)]
pub struct Options {
    #[clap(help = "ID of the domain")]
    pub domain: Option<String>,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    let domain_id = match options.domain {
        Some(id) => id,

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
