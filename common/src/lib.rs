use leptos::prelude::document;
pub use reactive_stores::Store;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, path::PathBuf};
use web_sys::wasm_bindgen::JsCast;

#[derive(Default, Clone, Debug)]
pub enum SelectedState {
    Copy,
    Cut,
    #[default]
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum UnitKind {
    Dirctory,
    Video,
    Audio,
    File,
}

impl Display for UnitKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let result = match self {
            UnitKind::Dirctory => "directory",
            UnitKind::File => "file",
            UnitKind::Video => "video",
            UnitKind::Audio => "audio",
        };
        write!(f, "{}", result)
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

    pub fn click_anchor(&self) {
        document()
            .get_element_by_id(&self.name())
            .unwrap()
            .unchecked_into::<web_sys::HtmlAnchorElement>()
            .click();
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

pub trait SortUnits {
    fn sort_units(&mut self);
}

impl SortUnits for Vec<Unit> {
    fn sort_units(&mut self) {
        self.sort_by_key(|x| (x.kind.clone(), x.name()));
    }
}

pub trait Retype {
    fn retype(&mut self);
}

impl Retype for Vec<Unit> {
    fn retype(&mut self) {
        self.iter_mut().for_each(|unit| {
            if unit.kind != UnitKind::File {
                return;
            }
            if let Some(x) = unit.path.extension().and_then(|x| x.to_str()) {
                if VIDEO_X.contains(&x) {
                    unit.kind = UnitKind::Video;
                } else if AUDIO_X.contains(&x) {
                    unit.kind = UnitKind::Audio;
                }
            };
        });
    }
}

#[derive(Default, Clone, Debug)]
pub struct Selected {
    pub units: Vec<Unit>,
    pub state: SelectedState,
}

impl Selected {
    pub fn clear(&mut self) {
        self.units.clear();
        self.none();
    }

    pub fn as_paths(&self) -> Vec<PathBuf> {
        self.units.iter().map(|x| x.path.clone()).collect()
    }

    pub fn has_dirs(&self) -> bool {
        self.units
            .iter()
            .any(|x| matches!(x.kind, UnitKind::Dirctory))
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
            self.units.sort_units();
        }
    }

    pub fn is_selected(&self, unit: &Unit) -> bool {
        self.units.contains(unit)
    }

    pub fn download_selected(self) {
        for unit in self.units.into_iter() {
            unit.click_anchor();
        }
    }
}

#[derive(Clone, Debug, Default, Store)]
pub struct GlobalState {
    select: Selected,
    current_path: PathBuf,
    media_play: Option<Unit>,
    units: Vec<Unit>,
    units_refetch_tick: bool,
    mkdir_state: Option<String>,
    password: bool,
}

impl GlobalState {
    pub fn new_store() -> Store<Self> {
        Store::new(Self::default())
    }
}
