use std::io::Write;

use tabwriter::TabWriter;

use super::types::Secret;

pub fn validate_name(name: &str) -> Result<(), String> {
    let regex = regex::Regex::new(r"^[a-zA-Z0-9_]{1,64}$").unwrap();

    if regex.is_match(name) {
        Ok(())
    } else {
        panic!("Invalid name. Secret names are limited to 64 characters in length, must be alphanumeric (with underscores) and are automatically uppercased.");
    }
}

pub fn format_secrets(secrets: &Vec<Secret>, title: bool) -> Vec<String> {
    let mut tw = TabWriter::new(vec![]);

    if title {
        writeln!(&mut tw, "{}", "NAME\tID\tCREATED").unwrap();
    }

    for secret in secrets {
        writeln!(
            &mut tw,
            "{}\t{}\t{}",
            secret.name, secret.id, secret.created_at
        )
        .unwrap();
    }

    String::from_utf8(tw.into_inner().unwrap())
        .unwrap()
        .lines()
        .map(|l| l.to_string())
        .collect()
}
