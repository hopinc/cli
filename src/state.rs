use std::io;

use crate::{config, store::auth::get_auth};
use reqwest::{Client, ClientBuilder};

pub struct State {
    pub client: Client,
    pub project: String,
    pub token: String,
}

pub struct StateOptions {
    pub override_project_id: Option<String>,
}

impl State {
    pub async fn new(options: StateOptions) -> io::Result<Self> {
        let client = ClientBuilder::new()
            .user_agent(format!("hop/{} on {}", config::VERSION, config::PLATFORM))
            .build()
            .unwrap();

        // do some logic to get current signed in user
        let _auth = get_auth().await;
        let token = "".into();

        // TODO: project subcommand group
        let project = if let Some(project_id) = options.override_project_id {
            project_id
        } else {
            "".into()
        };

        Ok(State {
            client,
            project,
            token,
        })
    }
}
