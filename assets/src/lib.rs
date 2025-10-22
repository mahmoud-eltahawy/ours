macro_rules! build_icon {
    ($name:ident,$path:literal) => {
        IconData {
            name: IconName::$name,
            data: include_bytes!($path),
        }
    };
}

pub const FAVICON: &[u8] = include_bytes!("../static/favicon.ico.gz");
pub const TAILWINDJS: &[u8] = include_bytes!("../static/tailwind.js.gz");
pub const HTMXJS: &[u8] = include_bytes!("../static/htmx.js.gz");

pub const ICONS: [IconData; 11] = [
    build_icon!(Folder, "../static/folder.svg"),
    build_icon!(File, "../static/file.svg"),
    build_icon!(Video, "../static/video.svg"),
    build_icon!(Audio, "../static/audio.svg"),
    build_icon!(Select, "../static/select.svg"),
    build_icon!(Close, "../static/close.svg"),
    build_icon!(Expand, "../static/expand.svg"),
    build_icon!(Collapse, "../static/collapse.svg"),
    build_icon!(Download, "../static/download.svg"),
    build_icon!(Home, "../static/home.svg"),
    build_icon!(Upload, "../static/upload.svg"),
];

pub fn get_icon(name: IconName) -> &'static [u8] {
    ICONS
        .into_iter()
        .find(|x| x.name == name)
        .unwrap_or_default()
        .data
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Default)]
pub enum IconName {
    Folder,
    File,
    Video,
    Audio,
    Select,
    Close,
    Expand,
    Collapse,
    Download,
    #[default]
    Home,
    Upload,
}

impl From<u8> for IconName {
    fn from(value: u8) -> Self {
        unsafe { std::mem::transmute(value) }
    }
}

impl From<IconName> for u8 {
    fn from(val: IconName) -> Self {
        unsafe { std::mem::transmute(val) }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Default)]
pub struct IconData {
    pub name: IconName,
    pub data: &'static [u8],
}

pub type Icon = &'static IconData;
