pub use icondata_core::IconData;

macro_rules! load {
    ($name:ident,$path:literal) => {
        pub const $name: IconData = IconData {
            data: include_str!($path),
            ..DEFAULT_ICON
        };
    };
}

load!(FOLDER_SVG, "../static/folder.svg");
load!(FILE_SVG, "../static/file.svg");
load!(VIDEO_SVG, "../static/video.svg");
load!(AUDIO_SVG, "../static/audio.svg");
load!(SELECT_SVG, "../static/select.svg");
load!(CLOSE_SVG, "../static/close.svg");
load!(EXPAND_SVG, "../static/expand.svg");
load!(COLLAPSE_SVG, "../static/collapse.svg");

pub const DEFAULT_ICON: icondata_core::IconData = icondata_core::IconData {
    style: None,
    x: None,
    y: None,
    width: None,
    height: None,
    view_box: Some("0 0 1024 1024"),
    stroke_linecap: None,
    stroke_linejoin: None,
    stroke_width: None,
    stroke: None,
    fill: None,
    data: "",
};
