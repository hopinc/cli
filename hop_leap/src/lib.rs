#![warn(clippy::pedantic)]
#![allow(
    clippy::missing_errors_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::unused_self
)]

pub mod errors;
pub mod manager;
pub(crate) mod runner;
pub(crate) mod shard;
