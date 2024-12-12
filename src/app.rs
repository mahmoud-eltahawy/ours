use crate::{Unit, Units};
use files_box::{ls, FilesBox};
use leptos::{ev, prelude::*};
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};
use media_player::MediaPlayer;
use nav_bar::NavBar;
use reactive_stores::Store;
use std::{collections::HashSet, path::PathBuf};

mod atoms;
mod files_box;
mod media_player;
mod nav_bar;

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[derive(Clone, Debug, Default, Store)]
struct GlobalState {
    selected: HashSet<Unit>,
    current_path: PathBuf,
    media_play: Option<Unit>,
    units: Vec<Unit>,
    units_refetch_tick: bool,
}

#[component]
pub fn App() -> impl IntoView {
    let store = Store::new(GlobalState::default());
    let ls_result = Resource::new(move || store.current_path().get(), ls);

    provide_meta_context();
    provide_context(store);

    Effect::new(move || {
        if let Some(mut xs) = ls_result.get().transpose().ok().flatten() {
            xs.retype();
            *store.units().write() = xs.resort();
        };
    });

    Effect::new(move || {
        let _ = store.units_refetch_tick().read();
        ls_result.refetch();
    });

    window_event_listener(ev::popstate, move |_| {
        store.selected().write().clear();
    });

    view! {
        <Stylesheet id="leptos" href="/pkg/webls.css"/>
        <Title text="Welcome to Leptos"/>
        <Router>
            <NavBar/>
            <main>
                <Routes fallback=|| "Page not found.">
                    <Route path=StaticSegment("") view=FilesBox/>
                </Routes>
            </main>
            <MediaPlayer/>
        </Router>
    }
}
