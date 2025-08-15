use axum::{Router, http::HeaderMap, response::Html, routing::get};
use axum_extra::response::{JavaScript, Wasm};

use site_assets::{FAVICON, INDEX, JS, WASM};

pub fn assets_router() -> Router {
    Router::new()
        .route("/", get(index))
        .route("/site.js", get(js))
        .route("/site_bg.wasm", get(wasm))
        .route("/favicon.ico", get(favicon))
}

fn gzip_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert("content-encoding", "gzip".parse().unwrap());
    headers
}

async fn index() -> (HeaderMap, Html<Vec<u8>>) {
    (gzip_headers(), Html(INDEX.into()))
}

async fn js() -> (HeaderMap, JavaScript<Vec<u8>>) {
    (gzip_headers(), JavaScript(JS.into()))
}

async fn wasm() -> (HeaderMap, Wasm<Vec<u8>>) {
    (gzip_headers(), Wasm(WASM.into()))
}

async fn favicon() -> (HeaderMap, Vec<u8>) {
    (gzip_headers(), FAVICON.into())
}
