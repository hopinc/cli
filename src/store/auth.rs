use std::collections::HashMap;

use super::utils::get_path;
use crate::config::AUTH_STORE_PATH;
use serde::{Deserialize, Serialize};
use tokio::{
    fs::{self, File},
    io::{AsyncReadExt, AsyncWriteExt},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Auth {
    authorized: HashMap<String, String>,
}

pub async fn save_auth(auth: Auth) -> Result<Auth, std::io::Error> {
    let path = get_path(AUTH_STORE_PATH);

    fs::create_dir_all(path.parent().unwrap())
        .await
        .expect("Failed to create auth store directory");

    match File::create(path).await {
        Ok(mut file) => {
            file.write(
                serde_json::to_string(&auth)
                    .expect("Failed to deserialize auth")
                    .as_bytes(),
            )
            .await
            .expect("Failed to write auth store");

            Ok(auth)
        }

        Err(err) => {
            eprintln!("Error creating auth file: {}", err);
            std::process::exit(1);
        }
    }
}

pub async fn get_auth() -> Auth {
    let path = get_path(AUTH_STORE_PATH);

    if fs::metadata(path.clone()).await.is_err() {
        save_auth(Auth {
            authorized: HashMap::new(),
        })
        .await
        .unwrap()
    } else {
        match File::open(path).await {
            Ok(mut file) => {
                let mut buffer = String::new();
                file.read_to_string(&mut buffer)
                    .await
                    .expect("Failed to read auth store");

                let auth: Auth = serde_json::from_str(&buffer).unwrap();
                auth
            }

            Err(err) => {
                eprintln!("Error opening auth file: {}", err);
                std::process::exit(1);
            }
        }
    }
}
