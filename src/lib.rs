use leptos::prelude::document;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, path::PathBuf};
use wasm_bindgen::JsCast;

#[cfg(feature = "ssr")]
use {
    std::{env::var, fs::canonicalize},
    tokio::{
        fs,
        io::{AsyncWriteExt, ErrorKind},
    },
};

// pub const EXTERNAL_NAME: &str = ;

pub mod app;
#[cfg(feature = "ssr")]
pub mod lsblk;

#[cfg(feature = "ssr")]
#[derive(Debug, Clone)]
pub struct ServerContext {
    pub root: PathBuf,
    pub port: u16,
    pub password: String,
}

#[cfg(feature = "ssr")]
impl ServerContext {
    pub async fn get() -> Self {
        let root = canonicalize(var("WEBLS_ROOT").unwrap()).unwrap();
        let port = var("WEBLS_PORT").unwrap().parse().unwrap();
        let password_path: PathBuf = var("WEBLS_PASSWORD").unwrap().parse().unwrap();
        let password = Self::get_password(password_path.clone()).await;
        Self {
            root,
            port,
            password,
        }
    }

    pub fn external_path(&self) -> PathBuf {
        let mut result = self.root.clone();
        result.push("external");
        result
    }

    async fn get_password(password_path: PathBuf) -> String {
        match fs::read_to_string(password_path.clone()).await {
            Ok(pass) => pass.trim().to_string(),
            Err(e) => match e.kind() {
                ErrorKind::NotFound => {
                    let password = "0000";
                    let mut file = fs::File::create(password_path).await.unwrap();
                    file.write_all(password.as_bytes()).await.unwrap();
                    password.to_string()
                }
                e => {
                    panic!("Error : {:#?}", e);
                }
            },
        }
    }
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
trait Retype {
    fn retype(&mut self);
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

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::*;
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}
