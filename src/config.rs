pub const PLATFORM: &str = std::env::consts::OS;
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// browser auth
pub const WEB_AUTH_URL: &str = "https://console.hop.io/cli-auth";
pub const PAT_FALLBACK_URL: &str = "https://console.hop.io/settings/pats";

// api stuff
pub const HOP_API_BASE_URL: &str = "https://api.hop.io/v1";
pub const HOP_BUILD_BASE_URL: &str = "https://derailed.hop.io/v1";
pub const HOP_REGISTRY_URL: &str = "registry.hop.io";

// store stuff
pub const AUTH_STORE_PATH: &str = ".hop/auth.json";
pub const CONTEXT_STORE_PATH: &str = ".hop/context.json";
