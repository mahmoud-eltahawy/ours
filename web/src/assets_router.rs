use axum::{Router, http::HeaderMap, routing::get};
use axum_extra::response::JavaScript;

pub fn assets_router() -> Router {
    Router::new()
        .route("/tailwind", get(tailwind))
        .route("/htmx", get(htmx))
        .route("/favicon.ico", get(favicon))
}

fn gzip_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert("content-encoding", "gzip".parse().unwrap());
    headers
}

async fn favicon() -> (HeaderMap, Vec<u8>) {
    pub const FAVICON: &[u8] = include_bytes!("../assets/favicon.ico.gz");
    (gzip_headers(), FAVICON.into())
}

async fn tailwind() -> (HeaderMap, JavaScript<Vec<u8>>) {
    pub const JS: &[u8] = include_bytes!("../assets/tailwind.js.gz");
    (gzip_headers(), JavaScript(JS.into()))
}

async fn htmx() -> (HeaderMap, JavaScript<Vec<u8>>) {
    pub const JS: &[u8] = include_bytes!("../assets/htmx.js.gz");
    (gzip_headers(), JavaScript(JS.into()))
}
