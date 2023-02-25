use std::path::PathBuf;

use anyhow::Result;
use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};

pub mod auth;
pub mod context;
pub mod hopfile;
pub mod macros;
pub mod utils;

pub trait Storable<T: Serialize + DeserializeOwned + Default + Clone = Self> {
    fn path() -> Result<PathBuf>;
}

#[async_trait]
pub trait Store<T: Storable + Serialize + DeserializeOwned + Default + Clone = Self> {
    // custom trait with its type to implement a macro easily
    #[allow(clippy::new_ret_no_self)]
    async fn new() -> Result<T>;
    async fn save(&self) -> Result<T>;
}
