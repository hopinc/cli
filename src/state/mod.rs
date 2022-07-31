pub mod http;
pub mod ws;

use std::str::FromStr;

use self::http::HttpClient;
use self::ws::WebsocketClient;
use crate::commands::auth::login::util::{token_options, TokenType};
use crate::store::auth::Auth;
use crate::store::context::Context;

#[derive(Debug)]
pub struct State {
    pub auth: Auth,
    pub ctx: Context,
    pub http: HttpClient,
    pub ws: WebsocketClient,
    token: Option<String>,
    pub token_type: Option<TokenType>,
}

pub struct StateOptions {
    pub override_project_id: Option<String>,
    pub override_token: Option<String>,
}

impl State {
    pub async fn new(options: StateOptions) -> Self {
        // do some logic to get current signed in user
        let auth = Auth::new().await;
        let mut ctx = Context::new().await;

        // override the project id if provided
        if options.override_project_id.is_some() {
            ctx.project_override = Some(options.override_project_id.unwrap());
        }

        // get the auth token from the auth store if it exists
        let init_token = match ctx.default_user {
            Some(ref user) => auth.authorized.get(user).map(|x| x.to_string()),
            None => None,
        };

        let (token, token_type) = Self::handle_token(init_token);

        // preffer the override token over the auth token
        let ws = WebsocketClient::new();
        let http = HttpClient::new(token.clone(), ctx.override_api_url.clone());

        State {
            token_type,
            token,
            http,
            auth,
            ctx,
            ws,
        }
    }

    /// Rebuilds the http client with the current auth token.
    fn handle_token(token: Option<String>) -> (Option<String>, Option<TokenType>) {
        let token = match token {
            Some(token) => Some(token),

            None => None,
        };

        let token_type = match token {
            // should only be PAT or PTK
            Some(ref token) => Some(
                TokenType::from_str(token.split('_').next().unwrap()).expect("Invalid token type"),
            ),
            None => None,
        };

        (token, token_type)
    }

    /// Login to the API
    pub async fn login(&mut self, token: Option<String>) {
        if token.is_none() && self.token.is_none() {
            panic!("No token provided");
        }

        if let Some(token) = token {
            let (token, token_type) = Self::handle_token(Some(token));

            self.token = token.clone();
            self.token_type = token_type;
            self.http = HttpClient::new(token, self.ctx.override_api_url.clone());
        }

        let response = token_options(self.http.clone(), self.token_type.clone()).await;

        // get current user to global
        self.ctx.current = Some(response.clone());

        self.ws.update_token(response.leap_token);
    }
}
