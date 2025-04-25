use std::{net::SocketAddr, path::PathBuf, time::Duration};

use app_error::{ServerError, ServerResult};
use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{get, post},
};
use get_port::Ops;
use tower_http::{services::ServeDir, timeout::TimeoutLayer};

pub mod app_error;
mod info;
mod mp4;
mod paste;

#[derive(Clone)]
struct Context {
    target_dir: PathBuf,
}

pub struct Server {
    site: PathBuf,
    target: PathBuf,
    port: Option<u16>,
    timeout: Duration,
}

impl Server {
    pub fn new(site: PathBuf, target: PathBuf) -> Self {
        let port = get_port::tcp::TcpPort::any("0.0.0.0");
        Self {
            site,
            target,
            port,
            timeout: Duration::from_secs(60 * 60),
        }
    }
    pub fn port(self, port: u16) -> Self {
        Self {
            port: Some(port),
            ..self
        }
    }
    pub fn timeout(self, timeout: Duration) -> Self {
        Self { timeout, ..self }
    }
    pub async fn serve(self) -> ServerResult<()> {
        let Self {
            site,
            target,
            port,
            timeout,
        } = self;
        let Some(port) = port else {
            return Err(ServerError::NonePort);
        };
        let addr = SocketAddr::from(([0, 0, 0, 0], port));

        let site_dir = ServeDir::new(&site);
        let target_dir = ServeDir::new(&target);
        let app = Router::new()
            .nest_service("/", site_dir)
            .nest_service("/download", target_dir)
            .route("/to/mp4", post(mp4::mp4_remux))
            .route("/upload", post(paste::upload))
            .route("/cp", post(paste::cp))
            .route("/mv", post(paste::mv))
            .route("/rm", post(paste::rm))
            .route("/disks", get(info::get_disks))
            .with_state(Context { target_dir: target })
            .layer(TimeoutLayer::new(timeout))
            .layer(DefaultBodyLimit::disable());

        let listener = tokio::net::TcpListener::bind(&addr).await?;
        axum::serve(listener, app).await?;
        Ok(())
    }
}
