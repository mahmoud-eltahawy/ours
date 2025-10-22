use leptos::prelude::*;

pub const CLOSE_PLAYER: &str = "/CLOSE_PLAYER";
pub const PLAYER_SECTION: &str = "PlayerSection";
pub const VIDEO_HREF: &str = "/videoplay";
pub const AUDIO_HREF: &str = "/audioplay";

impl VideoPlayerProps {
    pub fn to_html(self) -> String {
        VideoPlayer(self).to_html()
    }
}

#[component]
pub fn VideoPlayer(url: String) -> impl IntoView {
    view! {
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
    }
}
impl AudioPlayerProps {
    pub fn to_html(self) -> String {
        AudioPlayer(self).to_html()
    }
}

#[component]
pub fn AudioPlayer(url: String) -> impl IntoView {
    view! {
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
    }
}

impl HiddenPlayerProps {
    pub fn to_html(self) -> String {
        HiddenPlayer().to_html()
    }
}

#[component]
pub fn HiddenPlayer() -> impl IntoView {
    view! {
        <div id={PLAYER_SECTION} hidden></div>
    }
}
