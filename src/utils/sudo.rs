use anyhow::{Context, Result};
use tokio::process::Command;

pub async fn fix() -> Result<()> {
    // check if in sudo and user real user home
    if let Ok(user) = std::env::var("USER") {
        if user != "root" {
            return Ok(()); // not in sudo
        }

        if let Ok(user) = std::env::var("SUDO_USER") {
            log::debug!("Running as SUDO, using home of `{user}`");

            // running ~user to get home path
            let home = Command::new("sh")
                .arg("-c")
                .arg(format!("eval echo ~{}", user))
                .output()
                .await
                .with_context(|| format!("Failed to get home path of `{}`", user))?
                .stdout;

            let home = String::from_utf8(home)?;

            log::debug!("Setting home to `{}`", home);

            // set home path
            std::env::set_var("HOME", home.trim());
        } else {
            log::debug!("Running as root without sudo, using home `{user}`");
        }
    }

    Ok(())
}
