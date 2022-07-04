use crate::state::State;
use structopt::StructOpt;
use tokio::fs;

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
static DEFAULT_IGNORE: &[&str] = &[".git", "node_modules", "target"];

async fn compress(files: Vec<String>) -> Result<String, std::io::Error> {
    let path = std::env::temp_dir().join("hop_deployment.tar.gz");

    let file = fs::File::create(&path).await?;

    let mut archive = tokio_tar::Builder::new(file);
    archive.follow_symlinks(true);

    for file in files {
        archive.append_path(file).await?;
    }

    archive.finish().await?;

    Ok(path.to_str().unwrap().into())
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
            if !FILENAMES.contains(&filename) {
                continue;
            }

            hop_file = Some(entry.path().to_str().unwrap().to_string());
        }
    }

    println!("Found hop file: {}", hop_file.unwrap());

    let packed = compress(vec!["src/main.rs".into()])
        .await
        .expect("Could not compress files");

    println!("Packed to: {}", packed);

    todo!()
}
