#[cfg(all(
    not(feature = "native-tls"),
    not(feature = "rustls-tls-native-roots"),
    not(feature = "rustls-tls-webpki-roots")
))]
compile_error!("No TLS backend is enabled. Please enable one of the following backends: native-tls, rustls-tls-native-roots, rustls-tls-webpki-roots");

fn main() {}
