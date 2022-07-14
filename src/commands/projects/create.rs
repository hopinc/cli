use crate::{
    done,
    state::{http::HttpClient, State},
};
use serde::Serialize;
use structopt::StructOpt;

use super::types::{CreateResponse, ProjectRes};

#[derive(Debug, StructOpt)]
#[structopt(about = "Create a new project")]
pub struct CreateOptions {
    #[structopt(name = "namespace", help = "Namespace of the project")]
    namespace: String,
    #[structopt(name = "name", help = "Name of the project")]
    name: String,
    #[structopt(short = "d", long = "default", help = "Set as default project")]
    default: bool,
}

#[derive(Debug, Serialize)]
struct CreateParams {
    icon: Option<String>,
    name: String,
    namespace: String,
}

async fn create_project(
    params: CreateParams,
    http: HttpClient,
) -> Result<ProjectRes, std::io::Error> {
    let json = http
        .request::<CreateResponse>(
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

    done!("Created project `{}` ({})", options.name, options.namespace);

    Ok(())
}
