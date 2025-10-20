use axum::{extract, response::Html};
use leptos::prelude::*;

pub const CLOSE_PLAYER: &str = "/CLOSE_PLAYER";
pub const PLAYER_SECTION: &str = "PlayerSection";
pub const VIDEO_HREF: &str = "/videoplay";
pub const AUDIO_HREF: &str = "/audioplay";

pub async fn videoplayer(
    extract::Query(mut params): extract::Query<Vec<(usize, String)>>,
) -> Html<String> {
    params.sort_by_key(|x| x.0);
    let url = params
        .into_iter()
        .map(|(_, x)| x)
        .fold(String::from("/download"), |acc, x| acc + "/" + &x);

    let view = view! {
    <video
        id={PLAYER_SECTION}
        width="80%"
        class="fixed top-5 left-1/2 transform -translate-x-1/2"
        controls
        autoplay
        hx-get={CLOSE_PLAYER}
        hx-target="this"
        hx-swap="outerHTML"
        hx-trigger="pointerdown from:html"
    >
        <source src={url} type="video/mp4"/>
        Your browser does not support the video tag.
    </video>
    };

    Html(view.to_html())
}

pub async fn audioplayer(
    extract::Query(mut params): extract::Query<Vec<(usize, String)>>,
) -> Html<String> {
    params.sort_by_key(|x| x.0);
    let url = params
        .into_iter()
        .map(|(_, x)| x)
        .fold(String::from("/download"), |acc, x| acc + "/" + &x);

    let view = view! {
    <audio
        id={PLAYER_SECTION}
        class="fixed top-5 left-1/2 transform -translate-x-1/2"
        controls
        autoplay
        hx-get={CLOSE_PLAYER}
        hx-target="this"
        hx-swap="outerHTML"
        hx-trigger="pointerdown from:html"
    >
        <source src={url} type="audio/mp3"/>
        Your browser does not support the video tag.
    </audio>
    };

    Html(view.to_html())
}

#[component]
pub fn HiddenPlayer() -> impl IntoView {
    view! {
        <div id={PLAYER_SECTION} hidden></div>
    }
}

pub async fn close_player() -> Html<String> {
    Html(
        view! {
            <HiddenPlayer/>
        }
        .to_html(),
    )
}
