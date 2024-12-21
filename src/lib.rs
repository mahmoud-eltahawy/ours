use leptos::prelude::document;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use wasm_bindgen::JsCast;

pub mod app;

#[cfg(feature = "ssr")]
#[derive(Debug, Clone)]
pub struct ServerContext {
    pub root: PathBuf,
    pub password: String,
}

#[cfg(feature = "ssr")]
impl ServerContext {
    pub fn new(root: PathBuf, password: String) -> Self {
        Self { root, password }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum UnitKind {
    Dirctory,
    Video,
    Audio,
    File,
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
trait Units {
    fn retype(&mut self);
    fn resort(self) -> Self;
}

const VIDEO_X: [&str; 39] = [
    "webm", "mkv", "ts", "flv", "vob", "ogv", "ogg", "rrc", "gifv", "mng", "mov", "avi", "qt",
    "wmv", "yuv", "rm", "asf", "amv", "mp4", "m4p", "m4v", "mpg", "mp2", "mpeg", "mpe", "mpv",
    "m4v", "svi", "3gp", "3g2", "mxf", "roq", "nsv", "flv", "f4v", "f4p", "f4a", "f4b", "mod",
];

const AUDIO_X: [&str; 20] = [
    "wav", "mp3", "aiff", "raw", "flac", "alac", "ape", "wv", "tta", "aac", "m4a", "ogg", "opus",
    "wma", "au", "gsm", "amr", "ra", "mmf", "cda",
];

impl Units for Vec<Unit> {
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

    fn resort(self) -> Self {
        let (mut directories, mut files, mut videos, mut audios) =
            (Vec::new(), Vec::new(), Vec::new(), Vec::new());

        for unit in self.into_iter() {
            let target = match unit.kind {
                UnitKind::Dirctory => &mut directories,
                UnitKind::Video => &mut videos,
                UnitKind::Audio => &mut audios,
                UnitKind::File => &mut files,
            };
            target.push(unit);
        }

        [&mut directories, &mut videos, &mut audios, &mut files]
            .iter_mut()
            .for_each(|xs| xs.sort_by_key(|x| x.name()));

        directories
            .into_iter()
            .chain(videos)
            .chain(audios)
            .chain(files)
            .collect()
    }
}

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::*;
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}
