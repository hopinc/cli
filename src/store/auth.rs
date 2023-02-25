use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::fs::{self, File};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use super::utils::home_path;
use super::Storable;
use crate::impl_store;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Auth {
    pub authorized: HashMap<String, String>,
}

impl Storable for Auth {
    fn path() -> Result<PathBuf> {
        home_path(".hop/auth.json")
    }
}

impl_store!(Auth);
