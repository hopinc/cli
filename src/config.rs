pub const PLATFORM: &str = std::env::consts::OS;
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// browser
pub const WEB_AUTH_URL: &str = "https://console.hop.io/cli-auth";
pub const PAT_FALLBACK_URL: &str = "https://console.hop.io/settings/pats";
pub const WEB_DEPLOYMENTS_URL: &str = "https://console.hop.io/ignite/deployment/";

// api
pub const HOP_REGISTRY_URL: &str = "registry.hop.io";
pub const HOP_API_BASE_URL: &str = "https://api.hop.io/v1";
pub const HOP_BUILD_BASE_URL: &str = "https://builder.hop.io/v1";
pub const HOP_LEAP_EDGE_URL: &str = "wss://leap.hop.io/ws?encoding=json&compression=zlib";
pub const HOP_LEAP_EDGE_PROJECT_ID: &str = "project_MzA0MDgwOTQ2MDEwODQ5NzQ";

// store
pub const AUTH_STORE_PATH: &str = ".hop/auth.json";
pub const CONTEXT_STORE_PATH: &str = ".hop/context.json";
