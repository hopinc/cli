use std::env::temp_dir;
use std::path::PathBuf;

use crate::state::State;
use futures::stream::StreamExt;
use structopt::StructOpt;
use tokio::fs::{self, File};
use tokio_tar::Builder as TarBuilder;

#[derive(Debug, StructOpt)]
#[structopt(name = "deploy", about = "Deploy a project")]
pub struct DeployOptions {}

static FILENAMES: &[&str] = &[
    "hop.yml",
    "hop.yaml",
    "hop.json",
    ".hoprc",
    ".hoprc.yml",
    ".hoprc.yaml",
    ".hoprc.json",
];

// TODO: use this later
#[allow(dead_code)]
static DEFAULT_IGNORE: &[&str] = &[".git", ".gitignore", ".gitmodules"];

async fn compress(base_dir: PathBuf, ignore: Vec<&str>) -> Result<String, std::io::Error> {
    let archive_path = temp_dir().join("hop_deployment.tar.gz");
    let tar_file = File::create(&archive_path).await?;
    let ignore_list = [DEFAULT_IGNORE, FILENAMES, &ignore].concat();

    let mut archive = TarBuilder::new(tar_file);
    archive.follow_symlinks(true);

    let mut walker = async_walkdir::WalkDir::new(base_dir.clone());

    while let Some(entry) = walker.next().await {
        if let Ok(file) = entry {
            let relative = file.path().strip_prefix(&base_dir).unwrap().to_owned();

            if ignore_list.contains(&relative.to_str().unwrap().split("/").nth(0).unwrap()) {
                continue;
            }

            println!("{:?}", relative);

            archive.append_path(relative).await?;
        }
    }

    archive.finish().await?;

    Ok(archive_path.to_str().unwrap().into())
}

pub async fn handle_command(_options: DeployOptions, _state: State) -> Result<(), std::io::Error> {
    let path = std::env::current_dir().expect("Could not get current directory");

    // check if dir has a hop.yml hop.json file
    // if not, ask if they want to create one

    let mut dir = fs::read_dir(path.clone())
        .await
        .expect("Could not read directory");

    let mut hop_file: Option<String> = None;

    while let Some(entry) = dir.next_entry().await.expect("Could not read directory") {
        if let Some(filename) = entry.file_name().to_str() {
            println!("{}", filename);

            if !FILENAMES.contains(&filename) {
                continue;
            }

            hop_file = Some(entry.path().to_str().unwrap().to_string());
        }
    }

    println!("Found hop file: {}", hop_file.unwrap());

    let packed = compress(path, vec!["target", ".github"])
        .await
        .expect("Could not compress files");

    println!("Packed to: {}", packed);

    todo!()
}
