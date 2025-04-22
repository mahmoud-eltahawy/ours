use common::{GlobalState, GlobalStateStoreFields, Retype, SelectedState, SortUnits};
use files_box::{FilesBox, ls};
use leptos::{ev, html::Ol, prelude::*};
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    StaticSegment,
    components::{Route, Router, Routes},
};
use leptos_use::{
    UseDropZoneOptions, UseDropZoneReturn, use_drop_zone_with_options, use_event_listener,
    use_window,
};
use media_player::MediaPlayer;
use nav_bar::NavBar;

mod files_box;
mod media_player;
mod nav_bar;

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>

                <link href="video.css" rel="stylesheet"/>
                <script src="video.js" defer/>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <MetaTags />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    let store = GlobalState::new_store();
    let ls_result = Resource::new(move || store.current_path().get(), ls);

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
        <Stylesheet id="leptos" href="/pkg/webls.css" />
        <Title text="eltahawy's locker" />
        <Router>
            <NavBar files/>
            <main>
                <Routes fallback=|| "Page not found.">
                    <Route
                        path=StaticSegment("")
                        view=move || view! { <FilesBox drop_zone_el is_over_drop_zone /> }
                    />
                </Routes>
            </main>
            <MediaPlayer />
        </Router>
    }
}
