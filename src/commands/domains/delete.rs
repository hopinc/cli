use anyhow::Result;
use clap::Parser;

use super::util::{delete_domain, format_domains, get_all_domains};
use crate::{
    commands::{
        gateways::util::{format_gateways, get_all_gateways},
        ignite::util::{format_deployments, get_all_deployments},
    },
    state::State,
};

#[derive(Debug, Parser)]
#[clap(about = "Detach a domain from a gateway")]
pub struct Options {
    #[clap(name = "domain", help = "ID of the domain")]
    pub domain: Option<String>,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    let domain_id = match options.domain {
        Some(id) => id,

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
                .default(0)
                .items(&gateways_fmt)
                .interact_opt()?
                .expect("No gateways selected");

            let domains = get_all_domains(&state.http, &gateways[idx].id).await?;
            let domains_fmt = format_domains(&domains, false);

            let idx = dialoguer::Select::new()
                .with_prompt("Select a domain")
                .default(0)
                .items(&domains_fmt)
                .interact_opt()?
                .expect("No domains selected");

            domains[idx].id.clone()
        }
    };

    delete_domain(&state.http, &domain_id).await?;

    log::info!("Domain `{domain_id}` detached");

    Ok(())
}