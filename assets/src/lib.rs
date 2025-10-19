macro_rules! load {
    ($constname:ident,$name:literal,$path:literal) => {
        pub const $constname: IconData = IconData {
            name: $name,
            data: include_bytes!($path),
        };
    };
}

pub const FAVICON: &[u8] = include_bytes!("../static/favicon.ico.gz");
pub const TAILWINDJS: &[u8] = include_bytes!("../static/tailwind.js.gz");
pub const HTMXJS: &[u8] = include_bytes!("../static/htmx.js.gz");

load!(FOLDER_SVG, "folder.svg", "../static/folder.svg");
load!(FILE_SVG, "file.svg", "../static/file.svg");
load!(VIDEO_SVG, "video.svg", "../static/video.svg");
load!(AUDIO_SVG, "audio.svg", "../static/audio.svg");
load!(SELECT_SVG, "select.svg", "../static/select.svg");
load!(CLOSE_SVG, "close.svg", "../static/close.svg");
load!(EXPAND_SVG, "expand.svg", "../static/expand.svg");
load!(COLLAPSE_SVG, "collapse.svg", "../static/collapse.svg");
load!(DOWNLOAD_SVG, "download.svg", "../static/download.svg");
load!(HOME_SVG, "home.svg", "../static/home.svg");

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct IconData {
    pub name: &'static str,
    pub data: &'static [u8],
}

pub type Icon = &'static IconData;
