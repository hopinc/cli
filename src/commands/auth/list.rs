use clap::Parser;

use super::utils::format_users;
use crate::state::State;

#[derive(Debug, Parser)]
#[clap(about = "List all authenticated users")]
pub struct Options {
    #[clap(
        short = 'q',
        long = "quiet",
        help = "Only print the IDs of the authorized users"
    )]
    pub quiet: bool,
}

pub fn handle(options: &Options, state: &State) {
    let users = state.auth.authorized.keys().collect::<Vec<_>>();

    assert!(!users.is_empty(), "There are no authorized users");

    if options.quiet {
        let ids = users
            .iter()
            .map(|d| d.as_str())
            .collect::<Vec<_>>()
            .join(" ");

        print!("{}", ids);
    } else {
        let users_fmt = format_users(&users, true);

        println!("{}", users_fmt.join("\n"));
    }
}
