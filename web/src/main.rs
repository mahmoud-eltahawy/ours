use std::net::SocketAddr;

use axum::{
    Router,
    response::Html,
    routing::{get, post},
};
use leptos::prelude::*;

mod assets_router;
mod components;

#[tokio::main]
async fn main() {
    println!("Hello, world!");
    serve(8080).await
}

pub async fn serve(port: u16) {
    let addr = SocketAddr::from(([0; 4], port));

    let app = Router::new()
        .route("/", get(index))
        .route("/clicked", post(hello));
    let app = app.merge(assets_router::assets_router());

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

async fn index() -> Html<String> {
    use components::Index;
    let v = view! {
        <Index/>
    };
    // render_to_string();
    Html(v.to_html())
}

async fn hello() -> Html<String> {
    use components::Clicked;
    let v = view! {
        <Clicked/>
    };
    Html(v.to_html())
}
