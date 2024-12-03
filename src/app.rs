use crate::{Unit, UnitKind};
use leptos::prelude::*;
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    StaticSegment,
    components::{Route, Router, Routes},
};

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
pub async fn get_inner_files() -> Result<Vec<Unit>, ServerFnError> {
    use crate::{ServerContext, Unit, UnitKind};
    use tokio::fs;
    let context = use_context::<ServerContext>().unwrap();

    let mut dir = fs::read_dir(&context.root).await?;
    let mut paths = Vec::new();
    while let Some(x) = dir.next_entry().await? {
        let kind = if x.file_type().await?.is_dir() {
            UnitKind::Dirctory
        } else {
            UnitKind::File
        };
        let unit = Unit {
            path: x.path().strip_prefix(&context.root).unwrap().to_path_buf(),
            kind,
        };
        paths.push(unit);
    }

    Ok(paths)
}
#[component]
fn HomePage() -> impl IntoView {
    let paths = Resource::new(|| (), |_| get_inner_files());

    let paths_view = move || {
        paths.get().and_then(|x| x.ok()).map(|xs| {
            xs.into_iter()
                .map(|x| {
                    let path = x.path.to_str().unwrap().to_string();
                    view! {
                        <div class="grid grid-cols-1">
                            <UnitComp kind={x.kind}/>
                            <span>{path}</span>
                        </div>
                    }
                })
                .collect_view()
        })
    };

    view! {
        <Suspense fallback=|| "">
            <section class="flex flex-wrap gap-5 m-5 p-5">{paths_view}</section>
        </Suspense>
    }
}

#[component]
fn UnitComp(kind: UnitKind) -> impl IntoView {
    let icon_path = match kind {
        UnitKind::Dirctory => "directory.png",
        UnitKind::File => "file.png",
    };
    view! {
        <img src={icon_path} width=77/>
    }
}
