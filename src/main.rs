#[cfg(feature = "ssr")]
use {
    axum::Router,
    leptos::{logging::log, prelude::*},
    leptos_axum::{LeptosRoutes, generate_route_list},
    std::{env::var, fs::canonicalize, net::SocketAddr, path::PathBuf, time::Duration},
    tokio::{
        fs,
        io::{AsyncWriteExt, ErrorKind},
    },
    tower_http::{services::ServeDir, timeout::TimeoutLayer},
    webls::{ServerContext, app::*, lsblk},
};

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    let conf = get_configuration(None).unwrap();
    // let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(App);
    let webls_root = var("WEBLS_ROOT").unwrap();
    let port = var("WEBLS_PORT").unwrap().parse().unwrap();
    let password_path: PathBuf = var("WEBLS_PASSWORD").unwrap().parse().unwrap();
    let password = match fs::read_to_string(password_path.clone()).await {
        Ok(pass) => pass.trim().to_string(),
        Err(e) => match e.kind() {
            ErrorKind::NotFound => {
                let password = "0000";
                let mut file = fs::File::create(password_path).await.unwrap();
                file.write_all(password.as_bytes()).await.unwrap();
                password.to_string()
            }
            e => {
                panic!("Error : {:#?}", e);
            }
        },
    };
    let root = canonicalize(&webls_root).unwrap();
    let mut external_partitions = PathBuf::from(webls_root);
    external_partitions.push("external");
    let _ = lsblk::refresh_partitions((&external_partitions).into()).await;

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let serve_dir = ServeDir::new(root.clone());

    let app = Router::new()
        .leptos_routes_with_context(
            &leptos_options,
            routes,
            move || {
                let context = ServerContext::new(root.clone(), password.clone());
                provide_context(context);
            },
            {
                let leptos_options = leptos_options.clone();
                move || shell(leptos_options.clone())
            },
        )
        .layer(TimeoutLayer::new(Duration::from_secs(24 * 60 * 60)))
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptos_options)
        .nest_service("/download", serve_dir);

    log!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

#[cfg(not(feature = "ssr"))]
pub fn main() {}
