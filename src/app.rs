use std::path::PathBuf;

use leptos::{either::Either, logging::log, prelude::*};
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes, A},
    hooks::{use_navigate, use_query_map},
    StaticSegment,
};

use crate::{Unit, UnitKind};

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

// #[component]
// fn App() -> impl IntoView {
//     view! {
//     }
// }

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


        <Router>
            <nav>
                <A href="/">
                    <img class="m-5 w-12 hover:w-16" src="home.png"/>
                </A>
            </nav>
            <main>
                <Routes fallback=|| "Page not found.".into_view()>
                    <Route path=StaticSegment("") view=FilesBox/>
                </Routes>
            </main>
        </Router>
    }
}

#[component]
fn FilesBox() -> impl IntoView {
    let query = use_query_map();
    let get_pathbuf = move || {
        let queries = query.get();
        let mut i = 0;
        let mut result = PathBuf::new();
        while let Some(x) = queries.get(&i.to_string()) {
            result.push(x);
            i += 1;
        }
        result
    };

    let units = Resource::new(get_pathbuf, move |x| get_inner_files(x));

    Effect::new(move || {
        log!("{:#?}", get_pathbuf());
    });

    let units_view = move || {
        units.get().map(|xs| {
            let Ok(xs) = xs else {
                return Either::Left(());
            };
            let mut all = Vec::with_capacity(xs.len());
            let mut files = Vec::new();
            for x in xs.iter() {
                match x.kind {
                    UnitKind::Dirctory => all.push(x.clone()),
                    UnitKind::File => files.push(x.clone()),
                }
            }
            all.sort_by_key(|x| x.path.file_name().unwrap().to_str().unwrap().to_string());
            files.sort_by_key(|x| x.path.file_name().unwrap().to_str().unwrap().to_string());
            Either::Right(
                all.into_iter()
                    .chain(files)
                    .map(|unit| {
                        view! {
                            <UnitComp unit={unit.clone()}/>
                        }
                    })
                    .collect_view(),
            )
        })
    };

    view! {
        <Suspense fallback=|| "">
            <section class="flex flex-wrap gap-5 m-5 p-5">{units_view}</section>
        </Suspense>
    }
}

fn path_as_query(mut path: PathBuf) -> String {
    let mut list = Vec::new();
    while let Some(x) = path.file_name() {
        list.push(x.to_str().unwrap().to_string());
        path.pop();
    }
    //
    let mut result = Vec::new();
    for (i, x) in list.into_iter().rev().enumerate() {
        result.push(format!("{i}={x}"));
    }
    format!("/?{}", result.join("&&"))
}

#[component]
fn UnitComp(unit: Unit) -> impl IntoView {
    let navigate = use_navigate();
    let name = unit.name();
    let ondblclick = {
        let kind = unit.kind.clone();
        move |_| {
            if let UnitKind::Dirctory = kind {
                navigate(&path_as_query(unit.path.clone()), Default::default());
            }
        }
    };
    view! {
        <button
            on:dblclick=ondblclick
            class="grid grid-cols-1 hover:text-white hover:bg-black">
            <UnitIconComp kind={unit.kind}/>
            <span>{name}</span>
        </button>
    }
}

#[component]
fn UnitIconComp(kind: UnitKind) -> impl IntoView {
    let icon_path = match kind {
        UnitKind::Dirctory => "directory.png",
        UnitKind::File => "file.png",
    };
    view! {
        <img src={icon_path} width=77/>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    // Creates a reactive value to update the button
    let count = RwSignal::new(0);
    let on_click = move |_| *count.write() += 1;

    view! {
        <h1>"Welcome to Leptos!"</h1>
        <button on:click=on_click>"Click Me: " {count}</button>
    }
}

#[server]
pub async fn get_inner_files(base: PathBuf) -> Result<Vec<Unit>, ServerFnError> {
    use crate::{ServerContext, Unit, UnitKind};
    use tokio::fs;
    let context = use_context::<ServerContext>().unwrap();
    let root = context.root.join(base);
    let mut dir = fs::read_dir(&root).await?;
    let mut paths = Vec::new();
    while let Some(x) = dir.next_entry().await? {
        let kind = if x.file_type().await?.is_dir() {
            UnitKind::Dirctory
        } else {
            UnitKind::File
        };
        let unit = Unit {
            path: x.path().strip_prefix(&context.root)?.to_path_buf(),
            kind,
        };
        paths.push(unit);
    }

    Ok(paths)
}
