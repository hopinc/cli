use clap::Parser;
use serde::Serialize;

use super::types::{Project, SingleProjectResponse};
use crate::state::http::HttpClient;
use crate::state::State;

#[derive(Debug, Parser)]
#[clap(about = "Create a new project")]
pub struct CreateOptions {
    #[clap(name = "namespace", help = "Namespace of the project")]
    namespace: String,
    #[clap(name = "name", help = "Name of the project")]
    name: String,
    #[clap(short = 'd', long = "default", help = "Set as default project")]
    default: bool,
}

#[derive(Debug, Serialize)]
struct CreateParams {
    icon: Option<String>,
    name: String,
    namespace: String,
}

async fn create_project(params: CreateParams, http: HttpClient) -> Result<Project, std::io::Error> {
    let json = http
        .request::<SingleProjectResponse>(
            "POST",
            "/projects",
            Some((
                serde_json::to_string(&params).unwrap().into(),
                "application/json",
            )),
        )
        .await
        .expect("Error while creating project")
        .unwrap();

    Ok(json.project)
}

pub async fn handle_create(options: CreateOptions, mut state: State) -> Result<(), std::io::Error> {
    let params = CreateParams {
        name: options.name.clone(),
        namespace: options.namespace.clone(),
        icon: None,
    };

    let res = create_project(params, state.http.clone()).await?;

    if options.default {
        state.ctx.default_project = Some(res.id.clone());
        state.ctx.save().await?;
    }

    log::info!("Created project `{}` ({})", options.name, options.namespace);

    Ok(())
}
