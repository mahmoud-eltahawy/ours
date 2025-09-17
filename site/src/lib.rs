use std::path::PathBuf;
use std::sync::LazyLock;

use common::{GlobalState, GlobalStateStoreFields, SelectedState};
use common::{Unit, OS};
use delivery::Delivery;
use files_box::FilesBox;
use leptos::svg;
use leptos::{ev, prelude::*};
use leptos_meta::*;
use leptos_router::{components::*, StaticSegment};
use leptos_use::{use_event_listener, use_window};
use media_player::MediaPlayer;
use nav_bar::NavBar;

mod files_box;
mod media_player;
mod nav_bar;

pub static DELIVERY: LazyLock<Delivery> = LazyLock::new(|| {
    Delivery::new({
        #[cfg(debug_assertions)]
        let origin = format!("http://127.0.0.1:{}", include_str!("port.txt"));
        #[cfg(not(debug_assertions))]
        let origin = window().location().origin().unwrap();
        origin
    })
});

#[component]
pub fn App() -> impl IntoView {
    let store = GlobalState::new_store();
    let current_path = RwSignal::new(PathBuf::new());
    let ls_result = LocalResource::new(move || DELIVERY.clone().ls(current_path.get()));
    let host_os = LocalResource::new(move || DELIVERY.clone().get_host_os());
    let app_name = LocalResource::new(move || DELIVERY.clone().get_app_name());

    let show_download_link = Memo::new(move |_| {
        let ua = use_window()
            .navigator()
            .and_then(|x| x.user_agent().ok())
            .map(|x| x.to_lowercase());
        let name = app_name.get().and_then(|x| x.ok());
        let os = host_os.get().and_then(|x| x.ok());
        match (ua, os) {
            (Some(ua), Some(os)) if ua.contains(&os) => name,
            _ => None,
        }
    });

    let units = Memo::new(move |other| {
        let ls = match ls_result.get().transpose() {
            Ok(v) => v,
            Err(err) => {
                leptos::logging::error!("ls Error : {err}");
                return Vec::new();
            }
        };
        if ls.is_some() {
            ls.unwrap_or_default()
        } else {
            other.cloned().unwrap_or_default()
        }
    });

    provide_meta_context();
    provide_context(store);

    Effect::new(move || {
        let _ = store.units_refetch_tick().read();
        ls_result.refetch();
    });

    let _ = use_event_listener(use_window(), ev::popstate, move |_| {
        if let SelectedState::None = store.select().get().state {
            store.select().write().clear();
        }
    });

    let link = move || {
        show_download_link
            .get()
            .map(|x| DELIVERY.clone().url_path(&x))
    };
    view! {
        <Router>
            <NavBar current_path />
            <main>
                <ShowLet some={link} let:link>
                    <div>forget this shitty web app and download the native one by clicking <a href={link}>here</a></div>
                </ShowLet>
                <h1></h1>
                <Routes fallback=|| "Page not found.">
                    <Route
                        path=StaticSegment("")
                        view=move || view! { <FilesBox current_path units /> }
                    />
                </Routes>
            </main>
            <MediaPlayer />
        </Router>
    }
}

#[component]
pub fn Icon<I>(icon: I) -> impl IntoView
where
    I: Fn() -> assets::IconData + Send + Clone + 'static,
{
    move || {
        let icon = icon();
        svg::svg()
            .style(icon.style)
            .attr("x", icon.x)
            .attr("y", icon.y)
            .attr("width", "4em")
            .attr("height", "4em")
            .attr("viewBox", icon.view_box)
            .attr("stroke-linecap", icon.stroke_linecap)
            .attr("stroke-linejoin", icon.stroke_linejoin)
            .attr("stroke-width", icon.stroke_width)
            .attr("stroke", icon.stroke)
            .attr("fill", icon.fill.unwrap_or("currentColor"))
            .attr("role", "graphics-symbol")
            .inner_html(icon.data)
    }
}
