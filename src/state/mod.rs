pub mod http;
use anyhow::{ensure, Result};

use self::http::HttpClient;
use crate::commands::auth::login::util::{token_options, TokenType};
use crate::config::EXEC_NAME;
use crate::store::auth::Auth;
use crate::store::context::Context;

#[derive(Debug)]
pub struct State {
    pub is_ci: bool,
    pub auth: Auth,
    pub ctx: Context,
    pub http: HttpClient,
    token: Option<String>,
    token_type: Option<TokenType>,
}

pub struct StateOptions {
    pub override_project: Option<String>,
    pub override_token: Option<String>,
}

impl State {
    pub async fn new(options: StateOptions) -> Self {
        // do some logic to get current signed in user
        let auth = Auth::new().await;
        let mut ctx = Context::new().await;

        // override the project id if provided
        ctx.project_override = options
            .override_project
            .or_else(|| ctx.default_project.clone());

        // use the override token if provided
        let init_token = if let Some(override_token) = options.override_token {
            Some(override_token)
        // otherwise use the token from the store
        } else if let Some(ref user) = ctx.default_user {
            auth.authorized.get(user).map(|x| x.to_string())
        // if all fail then no token
        } else {
            None
        };

        let (token, token_type) = Self::handle_token(init_token);

        // preffer the override token over the auth token
        let http = HttpClient::new(
            token.clone(),
            std::env::var("API_URL")
                .ok()
                .or_else(|| ctx.override_api_url.clone()),
        );

        State {
            is_ci: Self::check_if_ci(),
            token_type,
            token,
            http,
            auth,
            ctx,
        }
    }

    /// Rebuilds the http client with the current auth token.
    fn handle_token(token: Option<String>) -> (Option<String>, Option<TokenType>) {
        let token_type = token
            .as_ref()
            .map(|token| TokenType::from_token(token).expect("Invalid token type"));

        (token, token_type)
    }

    /// Checks if the current environment is a CI environment.
    fn check_if_ci() -> bool {
        std::env::vars().any(|(key, _)| {
            matches!(
                key.as_str(),
                "BUILD_NUMBER"
                    | "CONTINUOUS_INTEGRATION"
                    | "GITLAB_CI"
                    | "CIRCLECI"
                    | "APPVEYOR"
                    | "RUN_ID"
                    | "CI"
            )
        })
    }

    /// Login to the API
    pub async fn login(&mut self, token: Option<String>) -> Result<()> {
        ensure!(
            token.is_some() || self.token.is_some(),
            "You are not logged in. Please run `{} auth login` first.",
            EXEC_NAME
        );

        if let Some(token) = token {
            let (token, token_type) = Self::handle_token(Some(token));

            self.token = token.clone();
            self.token_type = token_type;
            self.http = HttpClient::new(token, self.ctx.override_api_url.clone());
        }

        let response = token_options(self.http.clone(), self.token_type.clone()).await;

        if !response.email_verified {
            log::warn!("You need to verify your email address before you can use Hop.")
        }

        // get current user to global
        self.ctx.current = Some(response);

        // if the token is a ptk override the project
        if let Some(TokenType::Ptk) = self.token_type {
            self.ctx.project_override = self.ctx.current.as_ref().map(|cur| cur.id.clone())
        }

        Ok(())
    }

    pub fn token(&self) -> Option<String> {
        self.token.clone()
    }
}
