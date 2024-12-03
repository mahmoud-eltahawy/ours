use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};
use std::path::PathBuf;

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options islands=true/>
                <MetaTags/>
                <link rel="stylesheet" id="leptos" href="/pkg/leptos_tailwind.css"/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/webls.css"/>

        // sets the document title
        <Title text="Welcome to Leptos"/>

        // content for this welcome page
        <Router>
            <main>
                <Routes fallback=|| "Page not found.".into_view()>
                    <Route path=StaticSegment("") view=HomePage/>
                </Routes>
            </main>
        </Router>
    }
}

#[server]
pub async fn get_inner_files() -> Result<Vec<PathBuf>, ServerFnError> {
    use crate::ServerContext;
    use std::fs;
    let context = use_context::<ServerContext>().unwrap();

    let paths = fs::read_dir(&context.dir_path)
        .unwrap()
        .map(|x| x.unwrap().path())
        .collect::<Vec<_>>();

    Ok(paths)
}
#[component]
fn HomePage() -> impl IntoView {
    let paths = Resource::new(|| (), |_| get_inner_files());

    let paths_view = move || {
        paths.get().and_then(|x| x.ok()).map(|xs| {
            xs.into_iter()
                .map(|x| x.to_str().unwrap().to_string())
                .map(|x| {
                    view! {
                        <li>{x}</li>
                    }
                })
                .collect_view()
        })
    };

    view! {
        <Suspense fallback=|| "">
            <ol>{paths_view}</ol>
        </Suspense>
    }
}
