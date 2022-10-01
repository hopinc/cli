use anyhow::{bail, Ok, Result};

pub fn parse_restart_policy(policy: &Option<String>) -> Result<&'static str> {
    let result = match policy {
        Some(policy) => match policy.as_str() {
            "always" => "always",
            "unless-stopped" => "always",
            "on-failure" => "always",
            _ => {
                bail!("Unsupported restart policy: {}", policy);
            }
        },

        None => "never",
    };

    Ok(result)
}
