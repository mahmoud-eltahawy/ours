#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::Router;
    use leptos::logging::log;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use std::env::var;
    use std::fs::canonicalize;
    use std::net::SocketAddr;
    use tower_http::services::ServeDir;
    use webls::app::*;
    use webls::ServerContext;

    let conf = get_configuration(None).unwrap();
    // let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(App);
    let webls_root = var("WEBLS_ROOT").unwrap();
    let port = var("WEBLS_PORT").unwrap().parse().unwrap();
    let root = canonicalize(&webls_root).unwrap();

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let serve_dir = ServeDir::new(root.clone());

    let app = Router::new()
        .leptos_routes_with_context(
            &leptos_options,
            routes,
            move || {
                let context = ServerContext::new(root.clone());
                provide_context(context);
            },
            {
                let leptos_options = leptos_options.clone();
                move || shell(leptos_options.clone())
            },
        )
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
