use std::io;

use crate::config::{HOP_API_BASE_URL, PLATFORM, VERSION};
use crate::store::auth::Auth;
use crate::store::context::Context;
use reqwest::blocking::Client as BlockingClient;
use reqwest::header::HeaderMap;
use reqwest::Client as AsyncClient;

#[derive(Debug, Clone)]
pub struct HttpClient {
    pub client: AsyncClient,
    pub base_url: String,
    pub headers: HeaderMap,
    pub ua: String,
}

#[derive(Debug, Clone)]
pub struct State {
    pub http: HttpClient,
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

        headers.insert("content-type", "application/json".parse().unwrap());

        if token.is_some() {
            headers.insert("Authorization", token.clone().unwrap().parse().unwrap());
        }

        let ua = format!("hop_cli/{} on {}", VERSION, PLATFORM);

        HttpClient {
            headers: headers.clone(),
            ua: ua.clone(),
            client: AsyncClient::builder()
                .user_agent(ua.clone())
                .default_headers(headers.clone())
                .build()
                .unwrap(),
            base_url: HOP_API_BASE_URL.to_string(),
        }
    }

    pub fn sync_client(self) -> BlockingClient {
        BlockingClient::builder()
            .user_agent(self.http.ua)
            .default_headers(self.http.headers)
            .build()
            .unwrap()
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
        let token: Option<String> = match options.override_token {
            Some(token) => Some(token),

            None => {
                // get the auth token from the auth store if it exists
                match ctx.user {
                    Some(ref user) => auth.authorized.get(user).map(|x| x.to_string()),
                    None => None,
                }
            }
        };

        let client = Self::build_http_client(token.clone());

        Ok(State {
            ctx,
            http: client,
            auth,
        })
    }

    /// Rebuilds the http client with the current auth token.
    pub fn update_http_token(&mut self, token: String) {
        self.http = Self::build_http_client(Some(token));
    }
}
