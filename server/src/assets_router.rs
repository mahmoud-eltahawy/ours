use axum::http::HeaderMap;
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
