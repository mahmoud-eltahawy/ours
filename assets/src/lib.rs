macro_rules! include_svg {
    ($name:literal) => {
        include_bytes!(concat!("../static/", $name, ".svg"))
    };
}

pub const FAVICON: &[u8] = include_bytes!("../static/favicon.ico.gz");
pub const TAILWINDJS: &[u8] = include_bytes!("../static/tailwind.js.gz");
pub const HTMXJS: &[u8] = include_bytes!("../static/htmx.js.gz");

//WARNING : this implementation requires same sorting on ICONS assignment and IconName declration

pub const ICONS: [&[u8]; 11] = [
    include_svg!("folder"),
    include_svg!("file"),
    include_svg!("video"),
    include_svg!("audio"),
    include_svg!("select"),
    include_svg!("close"),
    include_svg!("expand"),
    include_svg!("collapse"),
    include_svg!("download"),
    include_svg!("home"),
    include_svg!("upload"),
];

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

impl IconName {
    pub fn get(self) -> &'static [u8] {
        let index: u8 = self.into();
        ICONS[index as usize]
    }
}
