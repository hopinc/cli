pub const ARCH: &str = std::env::consts::ARCH;
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(not(windows))]
pub const EXEC_NAME: &str = "hop";
#[cfg(windows)]
pub const EXEC_NAME: &str = "hop.exe";
pub const HOP_LEAP_PROJECT: &str = "project_MzA0MDgwOTQ2MDEwODQ5NzQ";
