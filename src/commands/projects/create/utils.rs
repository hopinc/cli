use std::convert::Infallible;

use anyhow::{Context, Result};
use console::Term;
use hyper::{Body, Request, Response};
use tokio::sync::mpsc::Sender;

use super::WEB_PAYMENTS_URL;
use crate::commands::auth::payment::utils::{format_payment_methods, get_all_payment_methods};
use crate::state::http::HttpClient;
use crate::utils::browser::listen_for_callback;
use crate::utils::urlify;

pub async fn get_payment_method_from_user(http: &HttpClient) -> Result<String> {
    loop {
        let payment_methods = get_all_payment_methods(http).await?;
        let mut payment_methods_fmt = format_payment_methods(&payment_methods, false)?;
        payment_methods_fmt.push("New payment method".to_string());

        let payment_method_idx = dialoguer::Select::new()
            .with_prompt("Select a payment method")
            .items(&payment_methods_fmt)
            .default(0)
            .interact()?;

        if payment_method_idx == payment_methods_fmt.len() - 1 {
            let _ = Term::stderr().clear_last_lines(1);

            let port = portpicker::pick_unused_port().with_context(|| {
                "Could not find an unused port. Please make sure you have at least one port available."
            })?;

            let url = format!(
                "{WEB_PAYMENTS_URL}?{}",
                vec!["callback", &format!("http://localhost:{port}/payment")].join("=")
            );

            if let Err(why) = webbrowser::open(&url) {
                log::error!("Could not open web a browser: {}", why);
                log::error!(
                    "Please open this URL in your browser: {}",
                    urlify(WEB_PAYMENTS_URL)
                );

                log::info!("Press enter to continue...");
                let _ = std::io::stdin().read_line(&mut String::new());
                Term::stderr().clear_last_lines(4)?;
            } else {
                log::info!("A browser window should have opened. If not, please open this URL in your browser: {}", urlify(&url));
                log::info!("Waiting for payment method to be created...");

                listen_for_callback(port, 10, |req, sender| {
                    Box::pin(request_handler(req, sender))
                })
                .await?;
            }

            // clear 3 because 2 logs and 1 new line
            let _ = Term::stdout().clear_last_lines(3);
        } else {
            return Ok(payment_methods[payment_method_idx].id.clone());
        }
    }
}

async fn request_handler(
    req: Request<Body>,
    sender: Sender<String>,
) -> Result<Response<Body>, Infallible> {
    let path = req.uri().path();

    // payment method created
    if path == "/payment" {
        sender.send("Payment method created".to_string()).await.ok();

        return Ok(Response::builder()
            .status(200)
            .body("Payment method created".into())
            .unwrap());
    }

    Ok(Response::builder()
        .status(400)
        .body("Bad request".into())
        .unwrap())
}
