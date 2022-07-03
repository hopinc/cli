use std::path::PathBuf;

pub fn get_path(to_join: &str) -> PathBuf {
    dirs::home_dir()
        .expect("Could not find `home` directory")
        .join(to_join)
}
