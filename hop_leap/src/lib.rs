#![warn(clippy::pedantic)]
#![allow(
    clippy::missing_errors_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::unused_self
)]

pub(crate) mod errors;
pub(crate) mod leap;
pub(crate) mod manager;
pub(crate) mod messenger;
pub(crate) mod runner;
pub(crate) mod shard;

pub use errors::*;
pub use leap::*;
