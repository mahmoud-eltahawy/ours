use std::path::PathBuf;

pub mod assets_router;
pub mod components;

#[derive(Clone)]
pub struct Context {
    pub target_dir: PathBuf,
}

pub const TAILWIND: &str = "/tailwind";
pub const HTMX: &str = "/htmx";
pub const FAVICON: &str = "/favicon.ico";
pub const BOXESIN: &str = "/boxesin";
