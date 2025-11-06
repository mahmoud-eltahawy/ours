pub const FAVICON: &[u8] = include_bytes!("../static/favicon.ico.gz");
pub const TAILWINDJS: &[u8] = include_bytes!("../static/tailwind.js.gz");
pub const HTMXJS: &[u8] = include_bytes!("../static/htmx.js.gz");

pub const ICONS_SIZE: usize = 14;

macro_rules! build_icons_defs {
    ($($name:ident);*) => {
        #[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
        pub enum IconName {
            $(
                $name,
            )*
        }

        pub const ICONS: [&[u8]; ICONS_SIZE] = [
            $(
                include_bytes!(concat!("../static/", stringify!($name), ".svg")),
            )*
        ];

    };
}

build_icons_defs!(Folder; File; Video; Audio; Select; Close; Expand; Collapse; Download; Home; Upload;Up;Down;Retry);

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
