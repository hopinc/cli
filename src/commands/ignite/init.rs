use std::{env::current_dir, path::PathBuf};

use super::util::{ContainerType, HopFile};
use crate::{done, state::State};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(about = "Initialize a new deployment in a directory")]
pub struct InitOptions {
    #[structopt(
        name = "d",
        help = "The directory to initialize, defaults to current directory"
    )]
    path: Option<PathBuf>,

    #[structopt(
        name = "n",
        long = "name",
        help = "The name of the deployment, defaults to the directory name"
    )]
    name: Option<String>,

    #[structopt(
        name = "t",
        long = "type",
        help = "The type of the deployment, defaults to `ephemeral`"
    )]
    d_type: Option<ContainerType>,
}

// FIXME: ignore errors for now
#[allow(unreachable_code, unused_variables)]

pub async fn handle_init(options: InitOptions, state: State) -> Result<(), std::io::Error> {
    let project = state.ctx.current_project_error().id;

    let mut dir = current_dir().expect("Could not get current directory");

    if let Some(path) = options.path {
        dir.push(path);
    }

    if HopFile::find(dir.clone()).await.is_some() {
        panic!("A hopfile already exists in this directory")
    }

    todo!("ask user for deployment details");

    let deployment = String::new();

    let hopfile = HopFile::new(dir.join("hop.yml"), project, deployment);

    hopfile
        .clone()
        .save()
        .await
        .expect("Could not save hopfile");

    done!("Created hopfile at {}", hopfile.path.unwrap().display());

    Ok(())
}
