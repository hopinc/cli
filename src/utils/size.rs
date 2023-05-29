use anyhow::{anyhow, Result};

// bytes have to be last because all other end with it

pub mod unit_multiplier {
    use anyhow::{bail, Result};

    // order matters here
    pub const BYTE_UNITS: [&str; 4] = ["GB", "MB", "KB", "B"];
    pub const B: u64 = 1;
    pub const KB: u64 = 1024;
    pub const MB: u64 = 1024 * 1024;
    pub const GB: u64 = 1024 * 1024 * 1024;

    pub fn from_str(unit: &str) -> Result<u64> {
        match unit {
            "GB" => Ok(GB),
            "MB" => Ok(MB),
            "KB" => Ok(KB),
            "B" => Ok(B),
            _ => bail!("Invalid unit: {unit}"),
        }
    }
}

pub fn parse_size(size: &str) -> Result<u64> {
    let mut size = size.trim().to_uppercase();

    // so stuff doesn't break
    if size.ends_with(['G', 'M', 'K']) {
        size = format!("{size}B");
    }

    let Some(unit) = unit_multiplier::BYTE_UNITS.iter().find(|unit| size.ends_with(&unit.to_string())) else {
        return Err(anyhow!("Invalid size unit: {size}"));
    };

    let Ok(size) = size[..size.len() - unit.len()].trim().parse::<u64>() else {
        return Err(anyhow!("Invalid size: {size}"));
    };

    Ok(size * unit_multiplier::from_str(unit)?)
}

pub fn user_friendly_size(size: u64) -> Result<String> {
    for unit in unit_multiplier::BYTE_UNITS {
        let factor = unit_multiplier::from_str(unit)?;

        if size < factor {
            continue;
        }

        return Ok(format!("{}{unit}", size / factor));
    }

    Ok(String::from("0B"))
}

// pub fn is_valid_mem_size(n: u64, min: u64, max: u64) -> bool {
//     n >= min && n <= max && n.is_power_of_two()
// }

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_size() {
        assert_eq!(parse_size("0B").unwrap(), 0);
        assert_eq!(parse_size("1B").unwrap(), 1);
        assert_eq!(parse_size("1KB").unwrap(), 1024);
        assert_eq!(parse_size("1MB").unwrap(), 1024 * 1024);
        assert_eq!(parse_size("1GB").unwrap(), 1024 * 1024 * 1024);
    }

    #[test]
    fn test_parse_size_invalid() {
        assert!(parse_size("1").is_err());
        assert!(parse_size("1TB").is_err());
        assert!(parse_size("1B1").is_err());
        assert!(parse_size("1B 1").is_err());
        assert!(parse_size("-1B").is_err());
    }

    #[test]
    fn test_user_friendly_size_uncommon() {
        assert_eq!(user_friendly_size(0).unwrap(), "0B");
        assert_eq!(user_friendly_size(1).unwrap(), "1B");
        assert_eq!(user_friendly_size(1024).unwrap(), "1KB");
        assert_eq!(user_friendly_size(1024 * 1024).unwrap(), "1MB");
        assert_eq!(user_friendly_size(1024 * 1024 * 1024).unwrap(), "1GB");

        assert_eq!(user_friendly_size(1024 * 1024 * 512).unwrap(), "512MB");
        assert_eq!(
            user_friendly_size(1024 * 1024 * 1024 * 512).unwrap(),
            "512GB"
        );
    }
}
