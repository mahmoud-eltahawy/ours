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
    pub async fn get(root: Option<PathBuf>, port: Option<u16>) -> Self {
        let root = root.unwrap_or(canonicalize(var("WEBLS_ROOT")
            .expect("if do not specify root in program arguments then need to specify it in environment varibale")).unwrap());
        let port = port.unwrap_or(var("WEBLS_PORT")
            .expect("if do not specify root in program arguments then need to specify it in environment varibale")
            .parse()
            .expect("port must be a number"));
        println!("ROOT = {:#?} \nPORT = {}", root, port);
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
