use std::path::PathBuf;
use std::sync::LazyLock;

use common::{GlobalState, GlobalStateStoreFields, SelectedState, SortUnits};
use common::{Retype, Unit};
use delivery::Delivery;
use files_box::{ls, FilesBox};
use leptos::html::Ol;
use leptos::svg;
use leptos::{ev, prelude::*};
use leptos_meta::*;
use leptos_router::{components::*, StaticSegment};
use leptos_use::{
    use_drop_zone_with_options, use_event_listener, use_window, UseDropZoneOptions,
    UseDropZoneReturn,
};
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
    let ls_result = LocalResource::new(move || ls(current_path.get()));

    let units = Memo::new(move |other| {
        let ls = match ls_result.get().transpose() {
            Ok(v) => v,
            Err(err) => {
                leptos::logging::error!("ls Error : {err}");
                return None;
            }
        };
        let result = ls
            .map(|mut xs| {
                xs.retype();
                xs
            })
            .map(|mut xs| {
                xs.sort_units();
                xs
            });
        if result.is_some() {
            result
        } else {
            other.cloned().flatten()
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
    let drop_zone_el = NodeRef::<Ol>::new();

    let UseDropZoneReturn {
        is_over_drop_zone,
        files,
    } = use_drop_zone_with_options(drop_zone_el, UseDropZoneOptions::default());

    view! {
        <Router>
            <NavBar files current_path/>
            <main>
                <Routes fallback=|| "Page not found.">
                    <Route
                        path=StaticSegment("")
                        view=move || view! { <FilesBox drop_zone_el is_over_drop_zone current_path units/> }
                    />
                </Routes>
            </main>
            <MediaPlayer />
        </Router>
    }
}

/// The Icon component.
#[component]
pub fn Icon(
    icon: RwSignal<icondata_core::IconData>,
    // #[prop(optional)] style: Option<&'static str>,
    // #[prop(optional)] width: Option<&'static str>,
    // #[prop(optional)] height: Option<&'static str>,
) -> impl IntoView {
    move || {
        let icon = icon.get();
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
