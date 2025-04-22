use common::Unit;

#[cfg(feature = "ssr")]
use {
    app::{App, shell},
    axum::Router,
    leptos::{logging::log, prelude::*},
    leptos_axum::{LeptosRoutes, generate_route_list},
    std::{env::var, fs::canonicalize, net::SocketAddr, path::PathBuf, time::Duration},
    tower_http::{services::ServeDir, timeout::TimeoutLayer},
};

pub mod app;

#[cfg(feature = "ssr")]
#[derive(Debug, Clone)]
pub struct ServerContext {
    pub root: PathBuf,
    pub port: u16,
}

#[cfg(feature = "ssr")]
impl ServerContext {
    pub async fn get(root: Option<PathBuf>, port: Option<u16>) -> Self {
        let root = root.unwrap_or(canonicalize(var("WEBLS_ROOT")
            .expect("if do not specify root in program arguments then need to specify it in environment varibale")).unwrap());
        let port = port.unwrap_or(var("WEBLS_PORT")
            .expect("if do not specify root in program arguments then need to specify it in environment varibale")
            .parse()
            .expect("port must be a number"));
        println!("ROOT = {:#?} \nPORT = {}", root, port);
        Self { root, port }
    }

    pub async fn refresh_partitions(&self) {
        let mut external = self.root.clone();
        external.push("external");
        let _ = partitions::refresh_partitions(external).await;
    }
}

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::*;
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}

#[cfg(feature = "ssr")]
pub async fn serve(root: Option<PathBuf>, port: Option<u16>) {
    let context = ServerContext::get(root.clone(), port).await;

    if let (None, None) = (root, port) {
        context.refresh_partitions().await;
    }

    let addr = SocketAddr::from(([0, 0, 0, 0], context.port));
    let serve_dir = ServeDir::new(context.root.clone());

    let conf = get_configuration(None).unwrap();
    let app = Router::new()
        .leptos_routes_with_context(
            &conf.leptos_options,
            generate_route_list(App),
            move || {
                provide_context(context.clone());
            },
            {
                let leptos_options = conf.leptos_options.clone();
                move || shell(leptos_options.clone())
            },
        )
        .layer(TimeoutLayer::new(Duration::from_secs(24 * 60 * 60)))
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(conf.leptos_options)
        .nest_service("/download", serve_dir);

    log!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
