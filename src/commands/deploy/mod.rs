use std::env::current_dir;

use super::ignite::util::{compress, HopFile};
use crate::state::State;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "deploy", about = "Deploy a project")]
pub struct DeployOptions {}

pub async fn handle_deploy(_options: DeployOptions, _state: State) -> Result<(), std::io::Error> {
    let path = current_dir().expect("Could not get current directory");

    // check if dir has a hop.yml hop.json file
    // if not, ask if they want to create one
    let hopfile = match HopFile::find(path.clone()).await {
        Some(hopfile) => {
            println!("Found hop file: {:?}", hopfile);
            hopfile
        }
        None => {
            panic!("No hopfile found, please run `hop ignite init` first");
        }
    };

    let packed = compress(hopfile.config.deployment, path)
        .await
        .expect("Could not compress files");

    println!("Packed to: {}", packed);

    todo!("push packed to remote server");
}
