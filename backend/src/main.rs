use common::{ServerContext, Unit, UnitKind};
use std::{env, fs::canonicalize, net::SocketAddr, path::PathBuf};
use tokio::{fs, net::TcpListener};

use axum::{extract::State, routing::post, Json, Router};
use tower_http::cors::{AllowHeaders, AllowMethods, AllowOrigin, CorsLayer};

#[tokio::main]
async fn main() {
    let Some(root) = env::args().nth(1).and_then(|x| canonicalize(&x).ok()) else {
        panic!("which directory i should target");
    };

    let context = ServerContext::new(root);

    let app = Router::new()
        .route("/files", post(get_inner_files))
        .with_state(context)
        .layer(
            CorsLayer::new()
                .allow_headers(AllowHeaders::any())
                .allow_origin(AllowOrigin::any())
                .allow_methods(AllowMethods::any()),
        );

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

pub async fn get_inner_files(
    context: State<ServerContext>,
    base: Json<Option<PathBuf>>,
) -> Json<Vec<Unit>> {
    let root = base.0.unwrap_or(context.root.clone());
    let mut dir = fs::read_dir(&root).await.unwrap();
    let mut paths = Vec::new();
    while let Some(x) = dir.next_entry().await.unwrap() {
        let kind = if x.file_type().await.unwrap().is_dir() {
            UnitKind::Dirctory
        } else {
            UnitKind::File
        };
        let unit = Unit {
            path: x.path().strip_prefix(&context.root).unwrap().to_path_buf(),
            kind,
        };
        paths.push(unit);
    }

    Json(paths)
}
