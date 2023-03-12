use std::str::FromStr;

use anyhow::{anyhow, Result};

// bytes have to be last because all other end with it
pub const BYTE_UNITS: [&str; 4] = ["GB", "MB", "KB", "B"];

#[derive(Debug)]
pub enum UnitMultiplier {
    B = 1,
    KB = 1024,
    MB = 1024 * 1024,
    GB = 1024 * 1024 * 1024,
}

impl FromStr for UnitMultiplier {
    type Err = anyhow::Error;

    fn from_str(u: &str) -> Result<Self, Self::Err> {
        match u.to_uppercase().as_str() {
            "B" => Ok(UnitMultiplier::B),
            "KB" => Ok(UnitMultiplier::KB),
            "MB" => Ok(UnitMultiplier::MB),
            "GB" => Ok(UnitMultiplier::GB),

            _ => Err(anyhow!("Invalid unit: {u}")),
        }
    }
}

pub fn parse_size(size: &str) -> Result<u64> {
    let mut size = size.trim().to_uppercase();

    if size.ends_with(['G', 'M', 'K']) {
        size = format!("{size}B");
    }

    let Some(unit) = BYTE_UNITS.iter().find(|unit| size.ends_with(&unit.to_string())) else {
        return Err(anyhow!("Invalid size unit: {size}"));
    };

    let Ok(size) = size[..size.len() - unit.len()].trim().parse::<u64>() else {
        return Err(anyhow!("Invalid size: {size}"));
    };

    Ok(size * UnitMultiplier::from_str(unit)? as u64)
}

// pub fn is_valid_mem_size(n: u64, min: u64, max: u64) -> bool {
//     n >= min && n <= max && n.is_power_of_two()
// }

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_size() {
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
}
