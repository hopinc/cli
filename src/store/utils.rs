use std::path::PathBuf;

pub fn home_path(to_join: &str) -> PathBuf {
    let path = dirs::home_dir()
        .expect("Could not find `home` directory")
        .join(to_join);

    log::debug!("Home path + joined: {:?}", path);

    path
}
