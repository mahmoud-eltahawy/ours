use axum::{extract::Path, http::HeaderMap};
use axum_extra::response::JavaScript;
use common::assets::{FAVICON, HTMXJS, TAILWINDJS};

fn gzip_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert("content-encoding", "gzip".parse().unwrap());
    headers
}

pub async fn favicon() -> (HeaderMap, Vec<u8>) {
    (gzip_headers(), FAVICON.into())
}

pub async fn tailwind() -> (HeaderMap, JavaScript<Vec<u8>>) {
    (gzip_headers(), JavaScript(TAILWINDJS.into()))
}

pub async fn htmx() -> (HeaderMap, JavaScript<Vec<u8>>) {
    (gzip_headers(), JavaScript(HTMXJS.into()))
}

pub async fn icon(Path(name): Path<String>) -> (HeaderMap, Vec<u8>) {
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "image/svg+xml".parse().unwrap());

    let b = match name.as_str() {
        "folder.svg" => common::assets::FOLDER_SVG.data.to_vec(),
        "file.svg" => common::assets::FILE_SVG.data.to_vec(),
        "video.svg" => common::assets::VIDEO_SVG.data.to_vec(),
        "audio.svg" => common::assets::AUDIO_SVG.data.to_vec(),
        "select.svg" => common::assets::SELECT_SVG.data.to_vec(),
        "close.svg" => common::assets::CLOSE_SVG.data.to_vec(),
        "expand.svg" => common::assets::EXPAND_SVG.data.to_vec(),
        "collapse.svg" => common::assets::COLLAPSE_SVG.data.to_vec(),
        "download.svg" => common::assets::DOWNLOAD_SVG.data.to_vec(),
        "home.svg" => common::assets::HOME_SVG.data.to_vec(),
        _ => Vec::new(),
    };
    (headers, b)
}
