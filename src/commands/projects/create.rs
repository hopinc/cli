use crate::{done, state::State};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

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

// types for the API response
#[derive(Debug, Deserialize)]
struct ProjectRes {
    pub id: String,
}

#[derive(Debug, Deserialize)]
struct CreateResponse {
    pub project: ProjectRes,
}

async fn create_project(params: CreateParams, state: State) -> Result<ProjectRes, std::io::Error> {
    let json = state
        .http
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

    let res = create_project(params, state.clone()).await?;

    done!("Created project `{}` ({})", options.name, options.namespace);

    if options.default {
        state.ctx.default_project = Some(res.id.clone());
        state.ctx.save().await?;
    }

    Ok(())
}
