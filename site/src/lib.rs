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

mod files_box;

#[component]
pub fn App() -> impl IntoView {
    let store = GlobalState::new_store();
    let ls_result = LocalResource::new(move || ls(store.current_path().get()));

    provide_meta_context();
    provide_context(store);

    Effect::new(move || {
        if let Some(mut xs) = ls_result.get().transpose().ok().flatten() {
            xs.retype();
            xs.sort_units();
            *store.units().write() = xs;
        };
    });

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
            // <NavBar files/>
            <main>
                <Routes fallback=|| "Page not found.">
                    <Route
                        path=StaticSegment("")
                        view=move || view! { <FilesBox drop_zone_el is_over_drop_zone /> }
                    />
                </Routes>
            </main>
            // <MediaPlayer />
        </Router>
    }
}
