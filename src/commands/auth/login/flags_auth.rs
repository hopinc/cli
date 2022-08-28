use super::types::{LoginRequest, LoginResponse};
use super::Options;
use crate::state::http::HttpClient;

pub async fn flags_login(options: Options, http: HttpClient) -> String {
    match options {
        Options { token: Some(_), .. }
        | Options {
            email: None,
            token: None,
            ..
        } => {
            if options.token.is_none() || options.token.as_ref().unwrap().is_empty() {
                dialoguer::Password::new()
                    .with_prompt("Token")
                    .interact()
                    .expect("Error getting token")
            } else {
                options.token.unwrap()
            }
        }

        Options {
            email: Some(username),
            password: None,
            ..
        }
        | Options {
            email: Some(username),
            password: Some(_),
            ..
        } => {
            let password =
                if options.password.is_none() || options.password.as_ref().unwrap().is_empty() {
                    dialoguer::Password::new()
                        .with_prompt("Password")
                        .interact()
                        .expect("Error getting password")
                } else {
                    options.password.unwrap()
                };

            login_with_credentials(http, username, password).await
        }

        // _ => panic!(
        //     "Invalid login options, run `{} auth login --help` for more info",
        //     EXEC_NAME
        // ),
    }
}

async fn login_with_credentials(http: HttpClient, email: String, password: String) -> String {
    http.request::<LoginResponse>(
        "POST",
        "/auth",
        Some((
            serde_json::to_vec(&LoginRequest { email, password })
                .unwrap()
                .into(),
            "application/json",
        )),
    )
    .await
    .expect("Error while logging in")
    .expect("Error while parsing login response")
    .token
}
