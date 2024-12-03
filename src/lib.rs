use std::path::PathBuf;

pub mod app;

#[derive(Debug, Clone)]
pub struct ServerContext {
    pub dir_path: PathBuf,
}

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_islands();
}
