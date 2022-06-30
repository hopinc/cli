use std::convert::Infallible;

use crate::config::WEB_AUTH_URL;
use crate::state::State;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use tokio::sync::mpsc::{channel, Sender};
use tokio::task;

async fn handler_request(
    req: Request<Body>,
    sender: Sender<String>,
) -> Result<Response<Body>, Infallible> {
    let token = req.uri().path()[1..].to_string();

    sender.send(token).await.unwrap();

    Ok(Response::new("You've been authorized".into()))
}

async fn web_auth(port: u16) -> Result<String, std::io::Error> {
    let (sender, mut receiver) = channel::<String>(1);

    let service = make_service_fn(move |_| {
        let sender = sender.clone();

        async move {
            Ok::<_, Infallible>(service_fn(move |req: Request<Body>| {
                handler_request(req, sender.clone())
            }))
        }
    });

    let address = ([127, 0, 0, 1], port).into();

    let server = Server::bind(&address).serve(service);

    let runtime = task::spawn(async {
        if let Err(error) = server.await {
            eprintln!("Server error: {}", error);
        }
    });

    let response = receiver.recv().await;

    runtime.abort();

    Ok(response.unwrap())
}

pub async fn handle_login(_state: State) -> Result<(), std::io::Error> {
    println!("login");

    let port = portpicker::pick_unused_port().unwrap();

    let callback_url = format!("http://localhost:{}/", port);
    let auth_url = format!(
        "{}?{}",
        WEB_AUTH_URL,
        querystring::stringify(vec![("callback", &callback_url)])
    );

    println!("{}", callback_url);

    // lunch a web server to handle the auth request
    let _token = if webbrowser::open(&auth_url).is_ok() {
        web_auth(port)
            .await
            .expect("Error while opening web browser")
    } else {
        "".to_string()
    };

    // TODO: api request to hop to authorize
    // the user and get user id

    Ok(())
}
