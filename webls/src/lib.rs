use common::Unit;

#[cfg(feature = "ssr")]
use std::{env::var, fs::canonicalize, path::PathBuf};

pub mod app;

#[cfg(feature = "ssr")]
#[derive(Debug, Clone)]
pub struct ServerContext {
    pub root: PathBuf,
    pub port: u16,
}

#[cfg(feature = "ssr")]
impl ServerContext {
    pub async fn get() -> Self {
        let root = canonicalize(var("WEBLS_ROOT").unwrap()).unwrap();
        let port = var("WEBLS_PORT").unwrap().parse().unwrap();
        Self { root, port }
    }

    pub async fn refresh_partitions(&self) {
        let mut external = self.root.clone();
        external.push("external");
        let _ = partitions::refresh_partitions(external).await;
    }
}

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::*;
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}
