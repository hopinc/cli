use std::{
    fmt::Display,
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};

use chrono::{DateTime, Utc};
use fern::colors::{Color, ColoredLevelConfig};
use log::{Level, LevelFilter};
use ms::{__to_string__, ms};
use serde::{de, Deserialize, Deserializer, Serialize};

pub fn set_hook() {
    // setup a panic hook to easily exit the program on panic
    std::panic::set_hook(Box::new(|panic_info| {
        // print the panic message
        let message = if let Some(message) = panic_info.payload().downcast_ref::<String>() {
            message.clone()
        } else if let Some(message) = panic_info.payload().downcast_ref::<&str>() {
            (*message).to_string()
        } else {
            format!("{:?}", panic_info)
        };

        // add some color
        log::error!("{}", message);

        if cfg!(debug_assertions) {
            log::debug!("{}", panic_info);
        }

        std::process::exit(1);
    }));

    ctrlc::set_handler(|| {
        // since dialoguer is annoying it
        // doesnt show the cursor when we exit
        // so lets do that manually
        let term = console::Term::stdout();
        term.show_cursor().ok();

        std::process::exit(0);
    })
    .ok();
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
                .level(log::LevelFilter::Info)
                .chain(
                    fern::Dispatch::new()
                        .filter(|metadata| {
                            // Reject messages with the `Error` log level.
                            metadata.level() != log::LevelFilter::Error
                        })
                        .chain(std::io::stderr()),
                )
                .chain(
                    fern::Dispatch::new()
                        .level(log::LevelFilter::Error)
                        .chain(std::io::stdout()),
                ),
        )
        .apply()
        .unwrap();
}

pub fn deserialize_from_str<'de, S, D>(deserializer: D) -> Result<S, D::Error>
where
    S: FromStr,
    S::Err: Display,
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    S::from_str(&s).map_err(de::Error::custom)
}

pub fn relative_time(date: DateTime<Utc>) -> String {
    let milis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
        - date.timestamp_millis() as u64;

    ms!(milis, true)
}

pub fn ask_question_iter<T>(prompt: &str, choices: &[T], override_default: Option<T>) -> T
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
        .interact()
        .expect("Failed to select");

    choices[choice].clone()
}
