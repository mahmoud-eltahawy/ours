use axum::{
    extract::Path,
    http::{HeaderMap, StatusCode},
};
use axum_extra::response::JavaScript;
use common::assets::{FAVICON, HTMXJS, ICONS_SIZE, IconName, TAILWINDJS};

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

pub async fn icon(Path(name): Path<u8>) -> (StatusCode, HeaderMap, Vec<u8>) {
    let mut headers = HeaderMap::new();
    if name as usize >= ICONS_SIZE {
        headers.insert("Content-Type", "text/plain".parse().unwrap());
        return (StatusCode::BAD_REQUEST, headers, "f*ck you sob!!".into());
    }
    headers.insert("Content-Type", "image/svg+xml".parse().unwrap());

    let data = IconName::from(name).get().to_vec();

    (StatusCode::OK, headers, data)
}
