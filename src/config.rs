pub const PLATFORM: &str = std::env::consts::OS;
pub const ARCH: &str = std::env::consts::ARCH;
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(not(windows))]
pub const EXEC_NAME: &str = "hop";

#[cfg(windows)]
pub const EXEC_NAME: &str = "hop.exe";
