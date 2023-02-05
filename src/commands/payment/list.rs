use anyhow::Result;
use clap::Parser;

use super::utils::{format_payment_methods, get_all_payment_methods};
use crate::state::State;

#[derive(Debug, Parser)]
#[clap(about = "List all payment methods")]
pub struct Options {
    #[clap(short, long, help = "Only print the IDs of the payment methods")]
    pub quiet: bool,
}

pub async fn handle(options: &Options, state: &State) -> Result<()> {
    let payment_methods = get_all_payment_methods(&state.http).await?;

    if options.quiet {
        let ids = payment_methods
            .iter()
            .map(|d| d.id.as_str())
            .collect::<Vec<_>>()
            .join(" ");

        println!("{ids}");
    } else {
        let payment_methods_fmt = format_payment_methods(&payment_methods, true)?;

        println!("{}", payment_methods_fmt.join("\n"));
    }

    Ok(())
}
