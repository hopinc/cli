#[macro_export]
macro_rules! done {
    () => ({ println!("\x1b[32m\x1b[1mdone\x1b[0m") });
    ($fmt:expr) => ({ println!(concat!("\x1b[32m\x1b[1mdone:\x1b[0m ",$fmt)) });
    ($fmt:expr, $($arg:tt)*) => ({ println!(concat!("\x1b[32m\x1b[1mdone:\x1b[0m " ,$fmt), $($arg)*) });
}

#[macro_export]
macro_rules! warn {
    () => ({ println!("\x1b[33m\x1b[1mwarning\x1b[0m") });
    ($fmt:expr) => ({ println!(concat!("\x1b[33m\x1b[1mwarning:\x1b[0m ",$fmt)) });
    ($fmt:expr, $($arg:tt)*) => ({ println!(concat!("\x1b[33m\x1b[1mwarning:\x1b[0m " ,$fmt), $($arg)*) });
}

#[macro_export]
macro_rules! info{
    () => ({ println!("\x1b[34m\x1b[1minfo\x1b[0m") });
    ($fmt:expr) => ({ println!(concat!("\x1b[34m\x1b[1minfo:\x1b[0m ",$fmt)) });
    ($fmt:expr, $($arg:tt)*) => ({ println!(concat!("\x1b[34m\x1b[1minfo:\x1b[0m " ,$fmt), $($arg)*) });
}

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
        eprintln!("\x1b[31m\x1b[1merror:\x1b[0m {}", message);

        if cfg!(debug_assertions) {
            eprintln!("Backtrace: {}", panic_info)
        }

        std::process::exit(1);
    }));
}
