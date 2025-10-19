use std::{env::home_dir, net::SocketAddr, path::PathBuf};

use axum::{Router, routing::get};

use crate::components::{IndexPage, boxes_in};

mod assets_router;
mod components;

#[tokio::main]
async fn main() {
    serve(8080, home_dir().unwrap()).await
}

#[derive(Clone)]
struct Context {
    base: PathBuf,
}

pub async fn serve(port: u16, base: PathBuf) {
    let addr = SocketAddr::from(([0; 4], port));

    let app = Router::new()
        .route("/", get(IndexPage::handle))
        .route("/boxesin", get(boxes_in))
        .with_state(Context { base });
    let app = app.merge(assets_router::assets_router());

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}
