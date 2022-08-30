#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions, clippy::unused_self)]

pub mod errors;
pub mod leap;
pub(crate) mod manager;
pub(crate) mod messenger;
pub(crate) mod runner;
pub(crate) mod shard;

pub use leap::{LeapEdge, LeapOptions};
