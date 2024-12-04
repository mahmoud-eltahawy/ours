use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ServerContext {
    pub root: PathBuf,
}

impl ServerContext {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UnitKind {
    Dirctory,
    File,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Unit {
    pub path: PathBuf,
    pub kind: UnitKind,
}

impl Unit {
    pub fn name(&self) -> String {
        self.path.file_name().unwrap().to_str().unwrap().to_string()
    }
}
