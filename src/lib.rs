use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub mod app;

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

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_islands();
}
