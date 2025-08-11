use std::{net::SocketAddr, path::PathBuf, time::Duration};

use app_error::{ServerError, ServerResult};
use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{any, post},
};
use cd::ws_ls;
use common::{CP_PATH, LS_PATH, MKDIR_PATH, MP4_PATH, MV_PATH, RM_PATH, UPLOAD_PATH};
use get_port::Ops;
use tower_http::{cors::CorsLayer, services::ServeDir, timeout::TimeoutLayer};

pub mod app_error;
mod assets_router;
mod cd;
mod mp4;

#[derive(Clone)]
struct Context {
    target_dir: PathBuf,
}

pub struct Server {
    target: PathBuf,
    port: Option<u16>,
    timeout: Duration,
}

impl Server {
    pub fn new(target: PathBuf) -> Self {
        let port = get_port::tcp::TcpPort::any("0.0.0.0");
        Self {
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
            target,
            port,
            timeout,
        } = self;
        let Some(port) = port else {
            return Err(ServerError::NonePort);
        };
        let addr = SocketAddr::from(([0, 0, 0, 0], port));

        let target_dir = ServeDir::new(&target);

        let app = Router::new()
            .route(MP4_PATH, post(mp4::mp4_remux))
            .route(UPLOAD_PATH, post(cd::upload))
            .route(CP_PATH, post(cd::cp))
            .route(MV_PATH, post(cd::mv))
            .route(RM_PATH, post(cd::rm))
            .route(LS_PATH, post(cd::ls))
            .route("/wls", any(ws_ls))
            .route(MKDIR_PATH, post(cd::mkdir))
            .nest_service("/download", target_dir)
            .with_state(Context { target_dir: target })
            .layer(TimeoutLayer::new(timeout))
            .layer(CorsLayer::permissive())
            .layer(DefaultBodyLimit::disable());

        let app = app.merge(assets_router::assets_router());

        let listener = tokio::net::TcpListener::bind(&addr).await?;
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await?;
        Ok(())
    }
}
