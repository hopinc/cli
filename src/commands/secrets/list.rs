use anyhow::Result;
use clap::Parser;

use crate::commands::secrets::types::Secrets;
use crate::commands::secrets::util::format_secrets;
use crate::state::State;

#[derive(Debug, Parser)]
#[structopt(about = "List all secrets")]
pub struct Options {
    #[structopt(
        short = 'q',
        long = "quiet",
        help = "Only print the IDs of the secrets"
    )]
    pub quiet: bool,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    let project_id = state.ctx.current_project_error().id;

    let secrets = state
        .http
        .request::<Secrets>(
            "GET",
            format!("/projects/{}/secrets", project_id).as_str(),
            None,
        )
        .await
        .expect("Error while getting project info")
        .unwrap()
        .secrets;

    if options.quiet {
        let ids = secrets
            .iter()
            .map(|d| d.id.as_str())
            .collect::<Vec<_>>()
            .join(" ");

        println!("{}", ids);
    } else {
        let secrets_fmt = format_secrets(&secrets, true);

        println!("{}", secrets_fmt.join("\n"));
    }

    Ok(())
}
