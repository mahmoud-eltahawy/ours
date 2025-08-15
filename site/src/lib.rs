use std::path::PathBuf;
use std::sync::LazyLock;

use common::Unit;
use common::{GlobalState, GlobalStateStoreFields, SelectedState};
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

pub static DELIVERY: LazyLock<Delivery> =
    LazyLock::new(|| Delivery::new(window().location().origin().unwrap()));

#[component]
pub fn App() -> impl IntoView {
    let store = GlobalState::new_store();
    let current_path = RwSignal::new(PathBuf::new());
    let ls_result = LocalResource::new(move || DELIVERY.clone().ls(current_path.get()));

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

    view! {
        <Router>
            <NavBar current_path/>
            <main>
                <Routes fallback=|| "Page not found.">
                    <Route
                        path=StaticSegment("")
                        view=move || view! { <FilesBox current_path units/> }
                    />
                </Routes>
            </main>
            <MediaPlayer />
        </Router>
    }
}

/// The Icon component.
#[component]
pub fn Icon<I>(
    icon: I,
    // #[prop(optional)] style: Option<&'static str>,
    // #[prop(optional)] width: Option<&'static str>,
    // #[prop(optional)] height: Option<&'static str>,
) -> impl IntoView
where
    I: Fn() -> assets::IconData + Send + Clone + 'static,
{
    move || {
        let icon = icon();
        svg::svg()
            // .style(match (style, icon.style) {
            //     (Some(a), Some(b)) => Some(format!("{b} {a}")),
            //     (Some(a), None) => Some(a.to_string()),
            //     (None, Some(b)) => Some(b.to_string()),
            //     _ => None,
            // })
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
