use std::{net::SocketAddr, path::PathBuf, time::Duration};

use app_error::{ServerError, ServerResult};
use axum::{
    Router,
    extract::{self, DefaultBodyLimit, State},
    http::StatusCode,
    response::Html,
    routing::{get, get_service, post},
};
use axum_extra::{TypedHeader, headers::UserAgent};
use common::{CP_PATH, LS_PATH, MKDIR_PATH, MP4_PATH, MV_PATH, RM_PATH, UPLOAD_PATH};
use get_port::Ops;
use tower_http::{cors::CorsLayer, services::ServeDir, timeout::TimeoutLayer};
use web::{
    BOXESIN, Context, FAVICON, HTMX, IndexPage, TAILWIND,
    media::{self, AUDIO_HREF, VIDEO_HREF},
    utils::{self, self_path},
};

use assets_router::{favicon, htmx, tailwind};

use crate::{
    assets_router::icon,
    web_local::{fetch_data, is_same_os},
};

pub mod app_error;
mod assets_router;
mod cd;
mod mp4;
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
            .route(MP4_PATH, post(mp4::mp4_remux))
            .route(UPLOAD_PATH, post(cd::upload))
            .route(CP_PATH, post(cd::cp))
            .route(MV_PATH, post(cd::mv))
            .route(RM_PATH, post(cd::rm))
            .route(LS_PATH, post(cd::ls))
            .route(MKDIR_PATH, post(cd::mkdir))
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

async fn fallback(
    TypedHeader(user_agent): TypedHeader<UserAgent>,
    State(Context { target_dir }): State<Context>,
    reqwest: extract::Request,
) -> (StatusCode, Html<String>) {
    let mut path = reqwest.uri().path().to_string();
    if path.starts_with('/') {
        path.remove(0);
    }
    let Ok(path) = path.parse::<PathBuf>();
    let path = target_dir.join(path);
    let mut page = IndexPage::new(target_dir, is_same_os(user_agent));
    match fetch_data(&mut page, path).await {
        Ok(_) => (StatusCode::OK, Html(page.render())),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Html(format!("<h2>internal error {err:#?}</h2>")),
        ),
    }
}

async fn self_executable(TypedHeader(user_agent): TypedHeader<UserAgent>) -> (StatusCode, Vec<u8>) {
    if !is_same_os(user_agent) {
        return (StatusCode::BAD_REQUEST, vec![]);
    }

    let Ok(contents) = tokio::fs::read(self_path()).await else {
        return (StatusCode::INTERNAL_SERVER_ERROR, vec![]);
    };

    (StatusCode::OK, contents)
}
