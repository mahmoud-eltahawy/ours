use std::{env::args, net::SocketAddr, path::PathBuf, sync::LazyLock, time::Duration};

use app_error::{ServerError, ServerResult};
use axum::{
    Json, Router,
    extract::DefaultBodyLimit,
    http::{HeaderMap, StatusCode},
    routing::{get, get_service, post},
};
// use cd::ws_ls;
use common::{CP_PATH, LS_PATH, MKDIR_PATH, MP4_PATH, MV_PATH, NAME, OS, RM_PATH, UPLOAD_PATH};
use get_port::Ops;
use tower_http::{cors::CorsLayer, services::ServeDir, timeout::TimeoutLayer};
use web::{
    BOXESIN, Context, FAVICON, HTMX, TAILWIND,
    assets_router::{favicon, htmx, tailwind},
    components::{IndexPage, boxes_in},
};

pub mod app_error;
mod cd;
mod mp4;

pub struct Server {
    target: PathBuf,
    port: Option<u16>,
    timeout: Duration,
}

static SELF_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    args()
        .next()
        .and_then(|x| x.parse::<PathBuf>().ok())
        .unwrap()
});
static APP_NAME: LazyLock<String> = LazyLock::new(|| {
    let name = SELF_PATH.file_name().unwrap().to_str().unwrap().to_string();
    println!("serving self at {name}");
    name
});

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
            .route(OS, get(os))
            .route(NAME, get(name))
            .route(MKDIR_PATH, post(cd::mkdir))
            .route(&format!("/{}", &*APP_NAME), get(self_executable))
            .route("/", get(IndexPage::handle))
            .route(TAILWIND, get(tailwind))
            .route(HTMX, get(htmx))
            .route(FAVICON, get(favicon))
            .route(BOXESIN, get(boxes_in))
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

async fn self_executable(headers: HeaderMap) -> (StatusCode, Vec<u8>) {
    let ua = headers.get("User-Agent").take_if(|user_agent| {
        user_agent
            .to_str()
            .is_ok_and(|x| x.to_lowercase().contains(std::env::consts::OS))
    });
    if ua.is_none() {
        return (StatusCode::BAD_REQUEST, vec![]);
    }

    let Ok(contents) = tokio::fs::read(&*SELF_PATH).await else {
        return (StatusCode::INTERNAL_SERVER_ERROR, vec![]);
    };

    (StatusCode::OK, contents)
}

async fn os() -> Json<&'static str> {
    Json(std::env::consts::OS)
}

async fn name() -> Json<&'static str> {
    Json(&APP_NAME)
}
