mod http;

use std::io;

use self::http::HttpClient;
use crate::commands::auth::types::UserMe;
use crate::store::auth::Auth;
use crate::store::context::Context;

#[derive(Debug, Clone)]
pub struct State {
    pub auth: Auth,
    pub ctx: Context,
    pub http: HttpClient,
}

pub struct StateOptions {
    pub override_project_id: Option<String>,
    pub override_token: Option<String>,
}

impl State {
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
                match ctx.default_user {
                    Some(ref user) => auth.authorized.get(user).map(|x| x.to_string()),
                    None => None,
                }
            }
        };

        let client = HttpClient::new(token, ctx.override_api_url.clone());

        Ok(State {
            ctx,
            http: client,
            auth,
        })
    }

    /// Rebuilds the http client with the current auth token.
    pub fn update_http_token(&mut self, token: String) {
        self.http = HttpClient::new(Some(token), self.ctx.override_api_url.clone());
    }

    pub async fn login(&mut self) {
        let response = self
            .http
            .request::<UserMe>("GET", "/users/@me", None)
            .await
            .expect("Error logging in, try running `hop auth login`")
            .unwrap();

        // get current user to global
        self.ctx.me = Some(response.clone());
    }
}
