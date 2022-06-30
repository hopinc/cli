use std::convert::Infallible;

use crate::config::{PAT_FALLBACK_URL, WEB_AUTH_URL};
use crate::state::State;
use crate::types::{Base, UsersMe};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use structopt::StructOpt;
use tokio::sync::mpsc::{channel, Sender};
use tokio::task;

#[derive(Debug, StructOpt)]
pub struct LoginOptions {
    #[structopt(long = "browserless", help = "Do not use a browser to login")]
    pub browserless: bool,
}

async fn request_handler(
    req: Request<Body>,
    sender: Sender<String>,
) -> Result<Response<Body>, Infallible> {
    let token = req.uri().path()[1..].to_string();

    // only send if it's an actual token
    if token.starts_with("pat_") {
        sender.send(token).await.unwrap();
    }

    Ok(Response::new("You've been authorized".into()))
}

async fn web_auth(port: u16) -> Result<String, std::io::Error> {
    let (sender, mut receiver) = channel::<String>(1);

    let timeouter = sender.clone();

    let timeout = task::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(120)).await;
        timeouter.send("timeout".to_string()).await.unwrap();
    });

    let service = make_service_fn(move |_| {
        let sender = sender.clone();

        async move {
            Ok::<_, Infallible>(service_fn(move |req: Request<Body>| {
                request_handler(req, sender.clone())
            }))
        }
    });

    let address = ([127, 0, 0, 1], port).into();

    let server = Server::bind(&address).serve(service);

    let runtime = task::spawn(async move {
        if let Err(error) = server.await {
            eprintln!("Server error: {}", error);
        }
        timeout.abort();
    });

    let response = receiver.recv().await;

    runtime.abort();

    if Some("timeout".to_string()) == response {
        panic!("Reached the 2 minute timeout");
    }

    Ok(response.unwrap())
}

pub async fn handle_login(options: LoginOptions, mut state: State) -> Result<(), std::io::Error> {
    let port = portpicker::pick_unused_port().unwrap();

    let callback_url = format!("http://localhost:{}/", port);
    let auth_url = format!(
        "{}?{}",
        WEB_AUTH_URL,
        querystring::stringify(vec![("callback", &callback_url)])
    );

    // lunch a web server to handle the auth request
    let token = if !options.browserless && webbrowser::open(&auth_url).is_ok() {
        println!("Opening browser to {}", auth_url);

        web_auth(port)
            .await
            .expect("Error while opening web browser")
    } else {
        if !options.browserless {
            println!("Could not open web a browser.");
            println!("Please provide a personal access token manually.");
            println!("You can create one at {}", PAT_FALLBACK_URL);
        }

        // falback to simpe input
        dialoguer::Password::new()
            .with_prompt("Enter your token")
            .interact()
            .unwrap()
    };

    // update the token assuming it's a valid PAT
    state.update_token(token.clone());

    // for sanity fetch the user info
    let request = state
        .client
        .http
        .get(format!("{}/users/@me", state.client.base_url))
        .send()
        .await;

    // some client error
    if request.is_err() {
        eprintln!("Error while getting user info: {}", request.unwrap_err());
        std::process::exit(1);
    }

    let response = request.unwrap();

    // if status code is not 200, then the token is probably invalid
    // or platform is down lol!
    if !response.status().is_success() {
        if response.status().is_client_error() {
            eprintln!("The provided token is invalid or expired");
        } else if response.status().is_server_error() {
            eprintln!("Unknown server error occured: {}", response.status());
        } else {
            eprintln!("Unknown error");
        }

        std::process::exit(1);
    }

    // parse the json
    let json = response
        .json::<Base<UsersMe>>()
        .await
        .expect("Error while parsing json");

    // output the login info
    println!(
        "Logged in as: \"{}\" ({})",
        json.data.user.username, json.data.user.email
    );

    // save the state
    state
        .auth
        .authorized
        .insert(json.data.user.id.clone(), token);
    state.auth.save().await?;

    state.ctx.user = Some(json.data.user.id);
    state.ctx.save().await?;

    Ok(())
}
