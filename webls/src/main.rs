#[cfg(feature = "ssr")]
use {
    axum::Router,
    leptos::{logging::log, prelude::*},
    leptos_axum::{LeptosRoutes, generate_route_list},
    std::{net::SocketAddr, time::Duration},
    tower_http::{services::ServeDir, timeout::TimeoutLayer},
    webls::{ServerContext, app::*},
};

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    let context = ServerContext::get().await;
    context.refresh_partitions().await;

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

#[cfg(not(feature = "ssr"))]
pub fn main() {}
