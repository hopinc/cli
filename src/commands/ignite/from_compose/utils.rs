use anyhow::{bail, Result};
use regex::Regex;

use super::types::Service;

// order services by their dependencies
// unsure of the accuracy of this algorithm but its fine for now
pub fn order_by_dependencies(services: &mut [(&String, &Service)]) {
    services.sort_by(|(a_name, a_service), (b_name, b_service)| {
        let a_depends_on = a_service.depends_on.clone();
        let b_depends_on = b_service.depends_on.clone();

        if a_depends_on.is_none() && b_depends_on.is_none() {
            return std::cmp::Ordering::Equal;
        }

        if a_depends_on.is_none() && b_depends_on.is_some() {
            return std::cmp::Ordering::Less;
        }

        if b_depends_on.is_none() {
            return std::cmp::Ordering::Greater;
        }

        let a_depends_on = a_depends_on.unwrap();
        let b_depends_on = b_depends_on.unwrap();

        if a_depends_on.contains(b_name) {
            return std::cmp::Ordering::Less;
        }

        if b_depends_on.contains(a_name) {
            return std::cmp::Ordering::Greater;
        }

        std::cmp::Ordering::Equal
    });
}

const DURATION_UNITS: [&str; 5] = ["us", "ms", "s", "m", "h"];

pub fn get_seconds_from_docker_duration(duration: &str) -> Result<u64> {
    let validate = Regex::new(&format!(r"^((\d*)({}))+$", DURATION_UNITS.join("|")))?;

    if !validate.is_match(duration) {
        bail!("Invalid duration: {duration}");
    }

    let regex = Regex::new(&format!(r"(\d+)({})", DURATION_UNITS.join("|")))?;

    let captures = regex.captures_iter(duration);

    let mut out: u64 = 0;

    for capture in captures {
        let value = capture.get(1).unwrap().as_str().parse::<u64>()?;
        let unit = capture.get(2).unwrap().as_str();

        let multiplier = match unit {
            "us" => 1,
            "ms" => 1000,
            "s" => 1000 * 1000,
            "m" => 1000 * 1000 * 60,
            "h" => 1000 * 1000 * 60 * 60,
            _ => bail!("Invalid unit: {unit}",),
        };

        out += value * multiplier;
    }

    Ok(out / 1000 / 1000)
}
