use axum::http::HeaderMap;
use axum_extra::response::JavaScript;

fn gzip_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert("content-encoding", "gzip".parse().unwrap());
    headers
}

pub async fn favicon() -> (HeaderMap, Vec<u8>) {
    pub const FAVICON: &[u8] = include_bytes!("../assets/favicon.ico.gz");
    (gzip_headers(), FAVICON.into())
}

pub async fn tailwind() -> (HeaderMap, JavaScript<Vec<u8>>) {
    pub const JS: &[u8] = include_bytes!("../assets/tailwind.js.gz");
    (gzip_headers(), JavaScript(JS.into()))
}

pub async fn htmx() -> (HeaderMap, JavaScript<Vec<u8>>) {
    pub const JS: &[u8] = include_bytes!("../assets/htmx.js.gz");
    (gzip_headers(), JavaScript(JS.into()))
}
