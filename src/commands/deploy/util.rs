use std::collections::HashMap;
use std::path::PathBuf;

use tokio::fs;

use crate::utils::parse_key_val;

pub async fn env_file_to_map(path: PathBuf) -> HashMap<String, String> {
    let mut env = HashMap::new();

    assert!(
        path.exists(),
        "Could not find .env file at {}",
        path.display()
    );

    let file = fs::read_to_string(path).await.unwrap();
    let lines = file.lines();

    for line in lines {
        // ignore comments
        if line.starts_with('#') {
            continue;
        }

        match parse_key_val(line) {
            Ok((key, value)) => {
                env.insert(key, value);
            }
            Err(e) => log::warn!("Failed to parse env file line: {}", e),
        }
    }

    env
}
