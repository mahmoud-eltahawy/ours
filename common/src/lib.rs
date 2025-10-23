pub use assets;
use std::{fmt::Display, net::IpAddr};

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

pub const VIDEO_X: [&str; 39] = [
    "webm", "mkv", "ts", "flv", "vob", "ogv", "ogg", "rrc", "gifv", "mng", "mov", "avi", "qt",
    "wmv", "yuv", "rm", "asf", "amv", "mp4", "m4p", "m4v", "mpg", "mp2", "mpeg", "mpe", "mpv",
    "m4v", "svi", "3gp", "3g2", "mxf", "roq", "nsv", "flv", "f4v", "f4p", "f4a", "f4b", "mod",
];

pub const AUDIO_X: [&str; 20] = [
    "wav", "mp3", "aiff", "raw", "flac", "alac", "ape", "wv", "tta", "aac", "m4a", "ogg", "opus",
    "wma", "au", "gsm", "amr", "ra", "mmf", "cda",
];
