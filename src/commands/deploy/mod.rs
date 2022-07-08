use super::ignite::util::{compress, find_hop_file};
use crate::state::State;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "deploy", about = "Deploy a project")]
pub struct DeployOptions {}

pub async fn handle_deploy(_options: DeployOptions, _state: State) -> Result<(), std::io::Error> {
    let path = std::env::current_dir().expect("Could not get current directory");

    // check if dir has a hop.yml hop.json file
    // if not, ask if they want to create one
    let hopfile = match find_hop_file(path.clone()).await {
        Some(hopfile) => {
            println!("Found hop file: {:?}", hopfile);
            hopfile
        }
        None => {
            println!("No hopfile found");
            todo!("ask user to create a hop file here");
        }
    };

    let packed = compress(hopfile.deployment, path, vec!["target"])
        .await
        .expect("Could not compress files");

    println!("Packed to: {}", packed);

    todo!()
}
