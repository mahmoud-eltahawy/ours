use common::{Unit, UnitKind};
use leptos::prelude::*;
use std::path::PathBuf;

async fn get_inner_files(base: Option<PathBuf>) -> Vec<Unit> {
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
    let base = RwSignal::new(None);
    let units = LocalResource::new(move || get_inner_files(base.get()));

    let units_view = move || {
        units.get().map(|xs| {
            xs.iter()
                .map(|unit| {
                    view! {
                        <UnitComp unit={unit.clone()} base/>
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

#[component]
fn UnitComp(unit: Unit, base: RwSignal<Option<PathBuf>>) -> impl IntoView {
    let name = unit.name();
    let ondblclick = {
        let kind = unit.kind.clone();
        move |_| {
            if let UnitKind::Dirctory = kind {
                *base.write() = Some(unit.path.clone())
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
    mount_to_body(|| view! { <App/> });
}
