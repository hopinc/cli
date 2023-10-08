use anyhow::Result;
use clap::Parser;

use super::utils::format_webhooks;
use crate::state::State;

#[derive(Debug, Parser)]
#[clap(about = "List webhooks")]
#[group(skip)]
pub struct Options {
    #[clap(short, long, help = "Only print the IDs")]
    pub quiet: bool,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    let project = state.ctx.current_project_error()?;

    let webhooks = state.hop.webhooks.get_all(&project.id).await?;

    if options.quiet {
        let ids = webhooks
            .iter()
            .map(|d| d.id.as_str())
            .collect::<Vec<_>>()
            .join(" ");

        println!("{ids}");
    } else {
        let webhooks_fmt = format_webhooks(&webhooks, true);

        println!("{}", webhooks_fmt.join("\n"));
    }

    Ok(())
}
