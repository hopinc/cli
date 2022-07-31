use clap::Parser;

use crate::state::State;

use super::utils::format_users;

#[derive(Debug, Parser)]
#[clap(about = "List all deployments")]
pub struct ListOptions {
    #[clap(
        short = 'q',
        long = "quiet",
        help = "Only print the IDs of the authorized users"
    )]
    pub quiet: bool,
}

pub async fn handle_list(options: ListOptions, state: State) -> Result<(), std::io::Error> {
    let users = state.auth.authorized.keys().collect::<Vec<_>>();

    if options.quiet {
        let ids = users
            .iter()
            .map(|d| d.as_str())
            .collect::<Vec<_>>()
            .join(" ");

        println!("{}", ids);
    } else {
        let users_fmt = format_users(&users, true);

        println!("{}", users_fmt.join("\n"));
    }

    Ok(())
}
