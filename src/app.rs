use files_box::{ls, FilesBox};
use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};
use nav_bar::NavBar;
use reactive_stores::Store;
use std::{collections::HashSet, path::PathBuf};

use crate::{Unit, UnitKind};

mod atoms;
mod files_box;
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
    media_play: Option<(String, UnitKind)>,
    ls_result: Vec<Unit>,
    ls_refetch_tick: bool,
}

impl GlobalState {
    fn new() -> Self {
        Self {
            selected: HashSet::new(),
            current_path: PathBuf::new(),
            media_play: None,
            ls_result: Vec::new(),
            ls_refetch_tick: true,
        }
    }
}

#[component]
pub fn App() -> impl IntoView {
    let store = Store::new(GlobalState::new());
    let ls_result = Resource::new(move || store.current_path().get(), ls);

    provide_meta_context();
    provide_context(store);

    Effect::new(move || {
        if let Some(xs) = ls_result.get().transpose().ok().flatten() {
            *store.ls_result().write() = xs;
        };
    });
    Effect::new(move || {
        let _ = store.ls_refetch_tick().get();
        ls_result.refetch();
    });

    window_event_listener(leptos::ev::popstate, move |_| {
        store.selected().write().clear();
    });

    view! {
        <Stylesheet id="leptos" href="/pkg/webls.css"/>
        <Title text="Welcome to Leptos"/>
        <Router>
            <NavBar/>
            <main>
                <Routes fallback=|| "Page not found.".into_view()>
                    <Route path=StaticSegment("") view={move ||view! {<FilesBox/>}}/>
                </Routes>
            </main>
        </Router>
    }
}
