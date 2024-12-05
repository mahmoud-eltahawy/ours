use files_box::FilesBox;
use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};
use nav_bar::NavBar;
use std::{collections::HashSet, path::PathBuf};

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

type Selected = RwSignal<HashSet<PathBuf>>;

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();
    let selected: Selected = RwSignal::new(HashSet::new());
    window_event_listener(leptos::ev::popstate, move |_| {
        selected.update(|xs| xs.clear());
    });

    provide_context(selected);

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/webls.css"/>

        // sets the document title
        <Title text="Welcome to Leptos"/>


        <Router>
            <NavBar/>
            <main>
                <Routes fallback=|| "Page not found.".into_view()>
                    <Route path=StaticSegment("") view=FilesBox/>
                </Routes>
            </main>
        </Router>
    }
}
