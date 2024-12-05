use std::path::PathBuf;

use super::{atoms::Icon, Selected};
use crate::{Unit, UnitKind};
use leptos::{either::Either, prelude::*};
use leptos_router::hooks::{use_navigate, use_query_map};

#[server]
async fn ls(base: PathBuf) -> Result<Vec<Unit>, ServerFnError> {
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

#[component]
pub fn FilesBox() -> impl IntoView {
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

    let units = Resource::new(get_pathbuf, move |x| ls(x));

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
    let selected = use_context::<Selected>().unwrap();

    let ondblclick = {
        let unit = unit.clone();
        move |_| {
            selected.update(|xs| xs.clear());
            if let UnitKind::Dirctory = unit.kind {
                navigate(&path_as_query(unit.path.clone()), Default::default());
            }
        }
    };
    let onclick = {
        let unit = unit.clone();
        move |_| {
            selected.update(|xs| {
                if !xs.insert(unit.clone()) {
                    xs.remove(&unit);
                };
            })
        }
    };

    view! {
        <button
            on:dblclick=ondblclick
            on:click=onclick
            class="grid grid-cols-1 hover:text-white hover:bg-black"
        >
            <UnitIconComp unit={unit}/>
            <span>{name}</span>
        </button>
    }
}

#[component]
fn UnitIconComp(unit: Unit) -> impl IntoView {
    let selected = use_context::<Selected>().unwrap();
    let is_selected = Memo::new({
        let unit = unit.clone();
        move |_| selected.read().contains(&unit)
    });
    let icon_name = move || match unit.kind {
        UnitKind::Dirctory => "directory.png",
        UnitKind::File => "file.png",
    };
    view! {
        <Icon name={icon_name()} active={move || !is_selected.get()} />
    }
}
