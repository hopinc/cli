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
}
