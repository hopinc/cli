pub const ARCH: &str = std::env::consts::ARCH;
pub const PLATFORM: &str = std::env::consts::OS;
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(not(windows))]
pub const EXEC_NAME: &str = "hop";
#[cfg(windows)]
pub const EXEC_NAME: &str = "hop.exe";
pub const LEAP_PROJECT: &str = "project_MzA0MDgwOTQ2MDEwODQ5NzQ";

#[cfg(windows)]
pub const DEFAULT_EDITOR: &str = "notepad.exe";
#[cfg(not(windows))]
pub const DEFAULT_EDITOR: &str = "vi";
