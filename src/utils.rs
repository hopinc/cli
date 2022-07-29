use fern::colors::{Color, ColoredLevelConfig};
use log::{Level, LevelFilter};

pub fn set_hook() {
    // setup a panic hook to easily exit the program on panic
    std::panic::set_hook(Box::new(|panic_info| {
        // print the panic message
        let message = if let Some(message) = panic_info.payload().downcast_ref::<String>() {
            message.to_owned()
        } else if let Some(message) = panic_info.payload().downcast_ref::<&str>() {
            message.to_string()
        } else {
            format!("{:?}", panic_info).to_string()
        };

        // add some color
        log::error!("{}", message);

        if cfg!(debug_assertions) {
            log::trace!("{}", panic_info)
        }

        std::process::exit(1);
    }));

    ctrlc::set_handler(|| {
        // since dialoguer is annoying it
        // doesnt show the cursor when we exit
        // so lets do that manually
        let term = console::Term::stdout();
        let _ = term.show_cursor();
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
        .chain(std::io::stdout())
        .apply()
        .unwrap();
}
