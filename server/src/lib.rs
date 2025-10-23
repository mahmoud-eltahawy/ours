use std::{net::SocketAddr, path::PathBuf, time::Duration};

use app_error::{ServerError, ServerResult};
use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{get, get_service},
};
use get_port::Ops;
use tower_http::{cors::CorsLayer, services::ServeDir, timeout::TimeoutLayer};
use web::{
    BOXESIN, Context, FAVICON, HTMX, TAILWIND,
    media::{self, AUDIO_HREF, VIDEO_HREF},
    utils::{self},
};

use assets_router::{favicon, htmx, tailwind};

use crate::{
    assets_router::icon,
    web_local::{fallback, self_executable},
};

pub mod app_error;
mod assets_router;
mod web_local;

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
        let addr = SocketAddr::from(([0; 4], port));

        let target_dir = ServeDir::new(&target);

        let app = Router::new()
            .route(&utils::app_name_url(), get(self_executable))
            .route("/", get(web_local::index_page))
            .route(TAILWIND, get(tailwind))
            .route("/icon/{name}", get(icon))
            .route(VIDEO_HREF, get(web_local::videoplayer))
            .route(AUDIO_HREF, get(web_local::audioplayer))
            .route(media::CLOSE_PLAYER, get(web_local::close_player))
            .route(HTMX, get(htmx))
            .route(FAVICON, get(favicon))
            .route(&format!("{}/{{down}}", BOXESIN), get(web_local::boxes_in))
            .fallback(get(fallback))
            .nest_service("/download", get_service(target_dir))
            .with_state(Context { target_dir: target })
            .layer(TimeoutLayer::new(timeout))
            .layer(CorsLayer::permissive())
            .layer(DefaultBodyLimit::disable());

        let listener = tokio::net::TcpListener::bind(&addr).await?;
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await?;
        Ok(())
    }
}
