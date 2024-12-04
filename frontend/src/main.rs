use common::{Unit, UnitKind};
use leptos::{logging::log, prelude::*};
use leptos_router::{
    components::{Route, Router, Routes, A},
    hooks::{use_navigate, use_params_map, use_query_map},
    path,
};
use std::{path::PathBuf, str::FromStr};

async fn get_inner_files(base: PathBuf) -> Vec<Unit> {
    let client = reqwest::Client::new();

    client
        .post("http://127.0.0.1:3000/files")
        .json(&base)
        .send()
        .await
        .unwrap()
        .json::<Vec<Unit>>()
        .await
        .unwrap()
}

#[component]
fn App() -> impl IntoView {
    view! {
        <Router>
            <nav>
                <A href="/">"Home"</A>
                <button on:click=move |_| {
                    //back function
                }>"back"</button>
            </nav>
            <main>
                <Routes fallback=|| "not found">
                    <Route path={path!("/")} view=FilesBox/>
                </Routes>
            </main>
        </Router>
    }
}

#[component]
fn FilesBox() -> impl IntoView {
    let query = use_query_map();
    let get_pathbuf = move || {
        let mut i = 0;
        let mut result = PathBuf::new();
        while let Some(x) = query.get().get(&i.to_string()) {
            result.push(x);
            i += 1;
        }
        result
    };
    let units = LocalResource::new(move || get_inner_files(get_pathbuf()));

    Effect::new(move || {
        log!("{:#?}", get_pathbuf());
    });

    let units_view = move || {
        units
            .get()
            .map(|xs| {
                let xs = xs.iter().collect::<Vec<_>>();
                let mut result = Vec::with_capacity(xs.len());
                let mut follows = Vec::new();
                for x in xs {
                    match x.kind {
                        UnitKind::Dirctory => result.push(x.clone()),
                        UnitKind::File => follows.push(x.clone()),
                    }
                }
                result.sort_by_key(|x| x.path.file_name().unwrap().to_str().unwrap().to_string());
                follows.sort_by_key(|x| x.path.file_name().unwrap().to_str().unwrap().to_string());
                result.extend(follows);
                result
            })
            .map(|xs| {
                xs.iter()
                    .map(|unit| {
                        view! {
                            <UnitComp unit={unit.clone()}/>
                        }
                    })
                    .collect_view()
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
        UnitKind::Dirctory => "public/directory.png",
        UnitKind::File => "public/file.png",
    };
    view! {
        <img src={icon_path} width=77/>
    }
}

fn main() {
    mount_to_body(App);
}
