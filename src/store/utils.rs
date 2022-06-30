use std::path::PathBuf;

pub fn get_path(to_join: &str) -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| panic!("Could not find \"home\" directory"))
        .join(to_join)
}
