use std::{
    env::{args, home_dir},
    net::SocketAddr,
    path::PathBuf,
    sync::LazyLock,
    time::Duration,
};

use app_error::{ServerError, ServerResult};
use axum::{
    Router,
    extract::{self, DefaultBodyLimit, Query, State},
    http::{HeaderMap, StatusCode},
    response::Html,
    routing::{get, get_service, post},
};
use axum_extra::{TypedHeader, headers::UserAgent};
use common::{
    AUDIO_X, CP_PATH, LS_PATH, MKDIR_PATH, MP4_PATH, MV_PATH, RM_PATH, UPLOAD_PATH, Unit, UnitKind,
    VIDEO_X,
};
use get_port::Ops;
use tokio::fs;
use tower_http::{cors::CorsLayer, services::ServeDir, timeout::TimeoutLayer};
use web::{
    BOXESIN, Context, FAVICON, HTMX, IndexPage, TAILWIND,
    media::{self, AUDIO_HREF, AudioPlayerProps, HiddenPlayerProps, VIDEO_HREF, VideoPlayerProps},
    utils,
};

use assets_router::{favicon, htmx, tailwind};

use crate::assets_router::icon;

pub mod app_error;
mod assets_router;
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
            .route("/", get(index_page))
            .route(TAILWIND, get(tailwind))
            .route("/icon/{name}", get(icon))
            .route(VIDEO_HREF, get(videoplayer))
            .route(AUDIO_HREF, get(audioplayer))
            .route(media::CLOSE_PLAYER, get(close_player))
            .route(HTMX, get(htmx))
            .route(FAVICON, get(favicon))
            .route(&format!("{}/{{down}}", BOXESIN), get(boxes_in))
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

pub async fn boxes_in(
    Query(mut params): Query<Vec<(usize, String)>>,
    extract::Path(down): extract::Path<String>,
    State(Context { target_dir }): State<Context>,
) -> Html<String> {
    params.sort_by_key(|x| x.0);
    let parent = params.into_iter().map(|(_, x)| x).collect::<PathBuf>();

    let units = ls(target_dir.clone(), parent.clone()).await.unwrap();

    let is_downloadable = down == "down";

    Html(
        web::BoxesProps {
            units,
            target_dir,
            parent,
            is_downloadable,
        }
        .to_html(),
    )
}

pub async fn fetch_data(page: &mut IndexPage) -> Result<(), Box<dyn std::error::Error>> {
    let units = ls(PathBuf::new(), home_dir().unwrap()).await?;
    page.units = units;
    Ok(())
}

async fn ls(target_dir: PathBuf, base: PathBuf) -> Result<Vec<Unit>, Box<dyn std::error::Error>> {
    let root = target_dir.join(base);
    let mut dir = fs::read_dir(&root).await?;
    let mut units = Vec::new();
    while let Some(x) = dir.next_entry().await? {
        let kind = if x.file_type().await?.is_dir() {
            UnitKind::Folder
        } else {
            let ex = x.path();
            let ex = ex.extension().and_then(|x| x.to_str());
            match ex {
                Some(ex) => {
                    if VIDEO_X.contains(&ex) {
                        UnitKind::Video
                    } else if AUDIO_X.contains(&ex) {
                        UnitKind::Audio
                    } else {
                        UnitKind::File
                    }
                }
                _ => UnitKind::File,
            }
        };
        let unit = Unit {
            path: x.path().to_path_buf(),
            kind,
        };
        units.push(unit);
    }
    units.sort_by_key(|x| (x.kind.clone(), x.name()));
    Ok(units)
}

async fn index_page(
    TypedHeader(user_agent): TypedHeader<UserAgent>,
    State(Context { target_dir }): State<Context>,
) -> Html<String> {
    let same_os = user_agent
        .as_str()
        .to_lowercase()
        .contains(std::env::consts::OS);
    let mut data = IndexPage::new(target_dir, same_os);
    fetch_data(&mut data).await.unwrap();
    Html(data.render())
}

async fn videoplayer(
    extract::Query(mut params): extract::Query<Vec<(usize, String)>>,
) -> Html<String> {
    params.sort_by_key(|x| x.0);
    let url = params
        .into_iter()
        .map(|(_, x)| x)
        .fold(String::from("/download"), |acc, x| acc + "/" + &x);

    Html(VideoPlayerProps { url }.to_html())
}

pub async fn close_player() -> Html<String> {
    Html(HiddenPlayerProps {}.to_html())
}

async fn audioplayer(
    extract::Query(mut params): extract::Query<Vec<(usize, String)>>,
) -> Html<String> {
    params.sort_by_key(|x| x.0);
    let url = params
        .into_iter()
        .map(|(_, x)| x)
        .fold(String::from("/download"), |acc, x| acc + "/" + &x);

    Html(AudioPlayerProps { url }.to_html())
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
