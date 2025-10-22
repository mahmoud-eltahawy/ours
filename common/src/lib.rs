pub use assets::{self, IconData};
use assets::{IconName, get_icon};
use serde::{Deserialize, Serialize};
use std::{fmt::Display, net::IpAddr, path::PathBuf};

pub const OS: &str = "/os";
pub const NAME: &str = "/name";
pub const LS_PATH: &str = "/ls";
pub const MKDIR_PATH: &str = "/mkdir";
pub const MP4_PATH: &str = "/to/mp4";
pub const UPLOAD_PATH: &str = "/upload";
pub const CP_PATH: &str = "/cp";
pub const MV_PATH: &str = "/mv";
pub const RM_PATH: &str = "/rm";

#[derive(Debug, Clone)]
pub struct Origin {
    pub ip: IpAddr,
    pub port: u16,
}

impl Display for Origin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self { ip, port } = self;
        write!(f, "http://{ip}:{port}")
    }
}

impl Origin {
    pub fn new(ip: IpAddr, port: u16) -> Self {
        Self { ip, port }
    }
}

#[derive(Default, Clone, Debug)]
pub enum SelectedState {
    Copy,
    Cut,
    #[default]
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum UnitKind {
    Folder,
    Video,
    Audio,
    File,
}

impl From<UnitKind> for IconName {
    fn from(value: UnitKind) -> Self {
        match value {
            UnitKind::Folder => Self::Folder,
            UnitKind::Video => Self::Video,
            UnitKind::Audio => Self::Audio,
            UnitKind::File => Self::File,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Unit {
    pub path: PathBuf,
    pub kind: UnitKind,
}

impl Unit {
    pub fn name(&self) -> String {
        self.path.file_name().unwrap().to_str().unwrap().to_string()
    }

    pub fn icon(&self) -> &'static [u8] {
        get_icon(IconName::from(self.kind.clone()))
    }
}

pub const VIDEO_X: [&str; 39] = [
    "webm", "mkv", "ts", "flv", "vob", "ogv", "ogg", "rrc", "gifv", "mng", "mov", "avi", "qt",
    "wmv", "yuv", "rm", "asf", "amv", "mp4", "m4p", "m4v", "mpg", "mp2", "mpeg", "mpe", "mpv",
    "m4v", "svi", "3gp", "3g2", "mxf", "roq", "nsv", "flv", "f4v", "f4p", "f4a", "f4b", "mod",
];

pub const AUDIO_X: [&str; 20] = [
    "wav", "mp3", "aiff", "raw", "flac", "alac", "ape", "wv", "tta", "aac", "m4a", "ogg", "opus",
    "wma", "au", "gsm", "amr", "ra", "mmf", "cda",
];

#[derive(Default, Clone, Debug)]
pub struct Selected {
    pub on: bool,
    pub units: Vec<Unit>,
    pub state: SelectedState,
}

impl Selected {
    pub fn clear(&mut self) {
        self.units.clear();
        self.none();
        self.on = false;
    }

    pub fn as_paths(&self) -> Vec<PathBuf> {
        self.units.iter().map(|x| x.path.clone()).collect()
    }

    pub fn has_dirs(&self) -> bool {
        self.units
            .iter()
            .any(|x| matches!(x.kind, UnitKind::Folder))
    }

    pub fn is_clear(&self) -> bool {
        self.units.is_empty()
    }

    pub fn copy(&mut self) {
        self.state = SelectedState::Copy;
    }

    pub fn cut(&mut self) {
        self.state = SelectedState::Cut;
    }

    pub fn none(&mut self) {
        self.state = SelectedState::None;
    }

    pub fn remove_unit(&mut self, unit: &Unit) {
        self.units.retain(|x| x != unit);
        if self.units.is_empty() {
            self.none();
        }
    }

    pub fn toggle_unit_selection(&mut self, unit: &Unit) {
        if self.units.contains(unit) {
            self.remove_unit(unit);
        } else {
            self.units.push(unit.clone());
        }
    }

    pub fn toggle_unit_alone_selection(&mut self, unit: &Unit) {
        if self.units.contains(unit) {
            self.units.clear();
        } else {
            self.units = vec![unit.clone()];
        }
    }

    pub fn is_selected(&self, unit: &Unit) -> bool {
        self.units.contains(unit)
    }
}
