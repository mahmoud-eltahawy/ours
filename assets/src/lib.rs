macro_rules! build_icon {
    ($name:literal,$path:literal) => {
        IconData {
            name: $name,
            data: include_bytes!($path),
        }
    };
}

pub const FAVICON: &[u8] = include_bytes!("../static/favicon.ico.gz");
pub const TAILWINDJS: &[u8] = include_bytes!("../static/tailwind.js.gz");
pub const HTMXJS: &[u8] = include_bytes!("../static/htmx.js.gz");

pub const ICONS: [IconData; 11] = [
    build_icon!("folder", "../static/folder.svg"),
    build_icon!("file", "../static/file.svg"),
    build_icon!("video", "../static/video.svg"),
    build_icon!("audio", "../static/audio.svg"),
    build_icon!("select", "../static/select.svg"),
    build_icon!("close", "../static/close.svg"),
    build_icon!("expand", "../static/expand.svg"),
    build_icon!("collapse", "../static/collapse.svg"),
    build_icon!("download", "../static/download.svg"),
    build_icon!("home", "../static/home.svg"),
    build_icon!("upload", "../static/upload.svg"),
];

pub fn get_icon(name: &str) -> &'static [u8] {
    ICONS
        .into_iter()
        .find(|x| x.name.starts_with(name))
        .unwrap_or_default()
        .data
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Default)]
pub struct IconData {
    pub name: &'static str,
    pub data: &'static [u8],
}

pub type Icon = &'static IconData;
