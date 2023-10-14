use std::io::Write;

use anyhow::Result;
use clap::Parser;
use tabwriter::TabWriter;

use super::utils::get_tiers;
use crate::commands::ignite::groups::utils::fetch_grouped_deployments;
use crate::commands::ignite::utils::get_storage;
use crate::state::State;

#[derive(Debug, Parser)]
#[clap(about = "Inspect a deployment")]
#[group(skip)]
pub struct Options {
    #[clap(help = "The ID or name of the deployment")]
    pub deployment: Option<String>,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    let mut deployment = if let Some(id_or_name) = options.deployment {
        state.get_deployment_by_name_or_id(&id_or_name).await?
    } else {
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

        deployments[idx].clone()
    };

    let (tiers, storage) = tokio::join!(
        get_tiers(&state.http),
        get_storage(&state.http, &deployment.id)
    );
    let (tiers, storage) = (tiers?, storage?);

    let mut tw = TabWriter::new(vec![]);

    writeln!(tw, "{} ({})", deployment.name, deployment.id)?;
    writeln!(tw, "  Metadata")?;
    writeln!(tw, "\tImage: {}", deployment.config.image.name)?;
    writeln!(tw, "\tCreated: {}", deployment.created_at)?;
    writeln!(
        tw,
        "\tContainers: {}/{}",
        deployment.container_count, deployment.target_container_count
    )?;
    writeln!(
        tw,
        "\tRestart Policy: {}",
        deployment.config.restart_policy.take().unwrap_or_default()
    )?;
    writeln!(
        tw,
        "\tUses ephemeral containers: {}",
        if deployment.is_ephemeral() {
            "Yes"
        } else {
            "No"
        }
    )?;
    writeln!(
        tw,
        "\tEntrypoint: {}",
        deployment
            .config
            .entrypoint
            .map(|s| serde_json::to_string(&s).unwrap())
            .unwrap_or_else(|| "None".to_string())
    )?;
    writeln!(
        tw,
        "\tCommand: {}",
        deployment
            .config
            .cmd
            .map(|s| serde_json::to_string(&s).unwrap())
            .unwrap_or_else(|| "None".to_string())
    )?;
    writeln!(tw, "  Resources")?;
    writeln!(
        tw,
        "\tTier: {}",
        deployment.config.resources.get_tier_name(&tiers)?
    )?;
    writeln!(
        tw,
        "\tVolume: {}",
        storage
            .volume
            .map(|s| s.to_string())
            .unwrap_or_else(|| "None".to_string())
    )?;
    writeln!(
        tw,
        "\tBuild Cache: {}",
        storage
            .build_cache
            .map(|s| s.to_string())
            .unwrap_or_else(|| "None".to_string())
    )?;

    tw.flush()?;

    print!("{}", String::from_utf8(tw.into_inner()?)?);

    Ok(())
}
