pub mod arisu;
pub mod browser;
pub mod size;
pub mod sudo;

use std::error::Error;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use console::style;
use fern::colors::{Color, ColoredLevelConfig};
use log::{Level, LevelFilter};
use ms::{__to_string__, ms};
use serde::Serialize;
use serde_json::Value;
use tokio::fs;

pub fn set_hook() {
    // setup a panic hook to easily exit the program on panic
    std::panic::set_hook(Box::new(|panic_info| {
        // print the panic message
        let message = if let Some(message) = panic_info.payload().downcast_ref::<String>() {
            message.clone()
        } else if let Some(message) = panic_info.payload().downcast_ref::<&str>() {
            (*message).to_string()
        } else {
            format!("{panic_info:?}")
        };

        // add some color
        log::error!("{message}");

        #[cfg(debug_assertions)]
        log::debug!("{panic_info}");

        std::process::exit(1);
    }));
}

pub fn clean_term() {
    let term = console::Term::stdout();

    // if the terminal is a tty, clear the screen and reset the cursor
    if term.is_term() {
        term.show_cursor().ok();
    }
}

pub fn logs(verbose: bool) {
    let colors = ColoredLevelConfig::new()
        .info(Color::BrightCyan)
        .error(Color::BrightRed)
        .warn(Color::BrightYellow)
        .debug(Color::BrightWhite);

    fern::Dispatch::new()
        .format(move |out, message, record| {
            let level = record.level();

            match level {
                Level::Debug => out.finish(format_args!(
                    "{} [{}]: {}",
                    colors.color(Level::Debug).to_string().to_lowercase(),
                    record.target(),
                    message
                )),

                level => out.finish(format_args!(
                    "{}: {}",
                    colors.color(level).to_string().to_lowercase(),
                    message
                )),
            }
        })
        .level(if verbose {
            LevelFilter::Debug
        } else {
            LevelFilter::Info
        })
        .chain(
            fern::Dispatch::new()
                .filter(|metadata| !matches!(metadata.level(), Level::Error | Level::Warn))
                .chain(std::io::stdout()),
        )
        .chain(
            fern::Dispatch::new()
                .level(log::LevelFilter::Error)
                .level(log::LevelFilter::Warn)
                .chain(std::io::stderr()),
        )
        .apply()
        .ok();
}

pub fn relative_time(date: DateTime<Utc>) -> String {
    let milis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
        - date.timestamp_millis();

    ms!(milis.unsigned_abs(), true)
}

pub fn ask_question_iter<T>(prompt: &str, choices: &[T], override_default: Option<T>) -> Result<T>
where
    T: PartialEq + Clone + Serialize + Default,
{
    let choices_txt: Vec<String> = choices
        .iter()
        .map(|c| serde_json::to_string(c).unwrap().replace('"', ""))
        .collect();

    let to_compare = match override_default {
        Some(override_default) => override_default,
        None => T::default(),
    };

    let choice = dialoguer::Select::new()
        .with_prompt(prompt)
        .default(choices.iter().position(|x| x == &to_compare).unwrap())
        .items(&choices_txt)
        .interact()?;

    Ok(choices[choice].clone())
}

#[cfg(windows)]
const SEPARATOR: &str = ";";

#[cfg(not(windows))]
const SEPARATOR: &str = ":";

pub async fn in_path(program: &str) -> bool {
    #[cfg(windows)]
    let program = &format!("{}.exe", program);

    let path = std::env::var("PATH").unwrap();
    let paths: Vec<&str> = path.split(SEPARATOR).collect();

    for path in paths {
        let to_try = format!("{path}/{program}");

        log::debug!("Checking if {to_try} exists");

        if fs::metadata(to_try).await.is_ok() {
            return true;
        }
    }

    false
}

pub fn urlify(s: &str) -> String {
    style(s).bold().underlined().to_string()
}

pub fn validate_json(json: &str) -> Result<Value> {
    serde_json::from_str::<serde_json::Value>(json).map_err(|e| anyhow!("Invalid JSON: {e}"))
}

pub fn validate_json_non_null(json: &str) -> Result<Value> {
    match validate_json(json)? {
        Value::Null => Err(anyhow!("JSON cannot be null")),
        value => Ok(value),
    }
}

pub fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

pub async fn is_writable(path: &PathBuf) -> bool {
    if fs::OpenOptions::new()
        .write(true)
        .create_new(false)
        .open(path)
        .await
        .is_ok()
    {
        fs::remove_file(path).await.ok();

        return true;
    }

    false
}

pub fn parse_key_val<T, U>(s: &str) -> core::result::Result<(T, U), Box<dyn Error>>
where
    T: std::str::FromStr,
    T::Err: Error + 'static,
    U: std::str::FromStr,
    U::Err: Error + 'static,
{
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid KEY=value: no `=` found in `{s}`"))?;

    Ok((s[..pos].parse::<T>()?, s[pos + 1..].parse::<U>()?))
}
