use std::path::PathBuf;

use common::{GlobalState, GlobalStateStoreFields, SelectedState, SortUnits};
use common::{Retype, Unit};
use files_box::{ls, FilesBox};
use leptos::html::Ol;
use leptos::{ev, prelude::*};
use leptos_meta::*;
use leptos_router::{components::*, StaticSegment};
use leptos_use::{
    use_drop_zone_with_options, use_event_listener, use_window, UseDropZoneOptions,
    UseDropZoneReturn,
};
use nav_bar::NavBar;

mod files_box;
mod nav_bar;

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
            // <MediaPlayer />
        </Router>
    }
}
