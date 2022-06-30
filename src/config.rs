pub const PLATFORM: &str = std::env::consts::OS;
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const WEB_AUTH_URL: &str = "https://console.hop.io";
pub const AUTH_STORE_PATH: &str = ".hop/auth.json";

// TODO: project subcommand group
pub const _CONTEXT_STORE_PATH: &str = ".hop/context.json";
