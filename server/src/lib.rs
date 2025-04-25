use std::{net::SocketAddr, path::PathBuf, time::Duration};

use axum::{Router, routing::post};
use tower_http::{services::ServeDir, timeout::TimeoutLayer};

pub mod app_error;
mod mp4;

#[derive(Clone)]
struct Context {
    root: PathBuf,
}

pub async fn serve(root: PathBuf, port: u16, timeout: Duration) {
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    let serve_dir = ServeDir::new(&root);
    let app = Router::new()
        .nest_service("/", serve_dir)
        .route("/to/mp4", post(mp4::mp4_remux))
        .with_state(Context { root })
        .layer(TimeoutLayer::new(timeout));

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
