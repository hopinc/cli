use std::io;

use crate::config::{HOP_API_BASE_URL, PLATFORM, VERSION};
use crate::store::auth::Auth;
use crate::store::context::Context;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Client, ClientBuilder};

#[derive(Debug, Clone)]
pub struct HttpClient {
    pub http: Client,
    pub base_url: String,
}

#[derive(Debug, Clone)]
pub struct State {
    pub client: HttpClient,
    pub auth: Auth,
    pub ctx: Context,
}

pub struct StateOptions {
    pub override_project_id: Option<String>,
    pub override_token: Option<String>,
}

impl State {
    fn build_http_client(token: Option<String>) -> HttpClient {
        let mut headers = HeaderMap::new();

        if token.is_some() {
            headers.insert(
                "Authorization",
                HeaderValue::from_str(&token.clone().unwrap()).unwrap(),
            );
        }

        HttpClient {
            http: ClientBuilder::new()
                .user_agent(format!("hop/{} on {}", VERSION, PLATFORM))
                .default_headers(headers)
                .build()
                .unwrap(),
            base_url: HOP_API_BASE_URL.to_string(),
        }
    }

    pub async fn new(options: StateOptions) -> io::Result<Self> {
        // do some logic to get current signed in user
        let auth = Auth::new().await;
        let mut ctx = Context::new().await;

        // override the project id if provided
        if options.override_project_id.is_some() {
            ctx.project = Some(options.override_project_id.unwrap());
        }

        // preffer the override token over the auth token
        let token: Option<String> = if let Some(token) = options.override_token {
            // TODO: add user id to context if token was overridden

            Some(token)
        } else {
            // get the auth token from the auth store if it exists
            if let Some(ref user) = ctx.user {
                auth.authorized.get(user).map(|x| x.to_string())
            } else {
                None
            }
        };

        let client = Self::build_http_client(token.clone());

        Ok(State { ctx, client, auth })
    }

    pub fn update_token(&mut self, token: String) {
        self.client = Self::build_http_client(Some(token));
    }
}
