use std::path::PathBuf;

use crate::{
    app::{
        atoms::{BaseIcon, Icon, IconSize},
        nav_bar::upload,
        GlobalState, GlobalStateStoreFields, SelectedState,
    },
    Unit, UnitKind,
};
use leptos::{either::Either, html::Section, prelude::*};
use leptos_router::hooks::{use_navigate, use_query_map};
use leptos_use::{
    use_drop_zone_with_options, UseDropZoneEvent, UseDropZoneOptions, UseDropZoneReturn,
};
use reactive_stores::Store;
use web_sys::{Blob, FormData, KeyboardEvent};

#[cfg(feature = "ssr")]
use {crate::ServerContext, tokio::fs};

#[server]
pub async fn ls(base: PathBuf) -> Result<Vec<Unit>, ServerFnError> {
    let context: ServerContext = use_context().unwrap();
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
    let store: Store<GlobalState> = use_context().unwrap();
    Effect::new(move || {
        let queries = query.read();
        let mut i = 0;
        let mut result = PathBuf::new();
        while let Some(x) = queries.get_str(&i.to_string()) {
            result.push(x);
            i += 1;
        }
        store.current_path().set(result);
    });

    let drop_zone_el = NodeRef::<Section>::new();
    let upload_action = Action::new_local(|data: &FormData| upload(data.clone().into()));

    let on_drop = move |ev: UseDropZoneEvent| {
        let current_path = store.current_path().read();
        if let Some(password) = store.password().get_untracked() {
            for file in ev.files {
                let data = FormData::new().unwrap();
                let path = current_path.join(file.name());
                let path = path.join(password.clone());
                data.append_with_blob(path.to_str().unwrap(), &Blob::from(file))
                    .unwrap();
                upload_action.dispatch_local(data);
            }
        };
    };

    let UseDropZoneReturn {
        is_over_drop_zone, ..
    } = use_drop_zone_with_options(drop_zone_el, UseDropZoneOptions::default().on_drop(on_drop));

    Effect::new(move || {
        if !upload_action.pending().get() {
            store.units_refetch_tick().update(|x| *x = !*x);
        }
    });

    view! {
        <section
            class="flex flex-wrap gap-5 m-5 p-5 border-2 border-black"
            node_ref={drop_zone_el}
        >
            <Mkdir />
            <For each=move || store.units().get() key=|x| x.path.clone() let:unit>
                <UnitComp unit=unit is_over_drop_zone/>
            </For>
            <UndropableWarn is_over_drop_zone/>
        </section>
    }
}

#[component]
pub fn UndropableWarn(is_over_drop_zone: Signal<bool>) -> impl IntoView {
    let store: Store<GlobalState> = use_context().unwrap();
    let unauthorizied = move || is_over_drop_zone.get() && store.password().get().is_none();
    view! {
        <Show when=unauthorizied>
            <div class="absolute top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2 z-20">
                <h2 class="text-5xl text-red-700 text-center leading-loose">you must be an admin to drop files</h2>
            </div>
        </Show>
    }
}

#[server]
pub async fn mkdir(target: PathBuf, password: String) -> Result<(), ServerFnError> {
    let context = use_context::<ServerContext>().unwrap();
    if password != context.password {
        return Err(ServerFnError::new("wrong password"));
    };
    let target = context.root.join(target);
    fs::create_dir(target).await?;
    Ok(())
}

#[component]
fn Mkdir() -> impl IntoView {
    let store: Store<GlobalState> = use_context().unwrap();
    let mkdir_state = store.mkdir_state();
    let value = RwSignal::new(String::new());

    let mkdir =
        Action::new(move |input: &(PathBuf, String)| mkdir(input.0.clone(), input.1.clone()));
    let enter = move |ev: KeyboardEvent| {
        if ev.key() == "Enter" {
            if let Some(password) = mkdir_state.get() {
                let path = store.current_path().get_untracked();
                let new_path = path.join(value.get_untracked());
                mkdir.dispatch((new_path, password));
                *mkdir_state.write() = None;
                value.write().clear();
            }
        }
    };

    Effect::new(move || {
        if !mkdir.pending().get() {
            store.units_refetch_tick().update(|x| *x = !*x);
        }
    });

    let when = move || mkdir_state.get().is_some();
    view! {
        <Show when=when>
            <button>
                <Icon src="directory" />
                <input
                    class="p-2 border-2 border-black text-2xl"
                    on:keypress=enter
                    type="text"
                    bind:value=value
                />
            </button>
        </Show>
    }
}

fn path_as_query(mut path: PathBuf) -> String {
    let mut list = Vec::new();
    while let Some(x) = path.file_name() {
        list.push(x.to_str().unwrap().to_string());
        path.pop();
    }
    let result = list
        .into_iter()
        .rev()
        .enumerate()
        .map(|(i, x)| format!("{i}={x}"))
        .collect::<Vec<_>>()
        .join("&&");
    format!("/?{}", result)
}

#[component]
fn UnitComp(unit: Unit, is_over_drop_zone: Signal<bool>) -> impl IntoView {
    let navigate = use_navigate();
    let store = use_context::<Store<GlobalState>>().unwrap();

    let ondblclick = {
        let unit = unit.clone();
        move |_| match &unit.kind {
            UnitKind::Dirctory => {
                if matches!(store.select().read().state, SelectedState::None) {
                    store.select().write().clear();
                }
                navigate(&path_as_query(unit.path.clone()), Default::default());
            }
            UnitKind::Video | UnitKind::Audio => {
                *store.media_play().write() = Some(unit.clone());
            }
            UnitKind::File => {
                unit.click_anchor();
                store.select().write().remove_unit(&unit);
            }
        }
    };

    let onclick = {
        let unit = unit.clone();
        move |_| {
            store.select().write().toggle_unit_selection(&unit);
        }
    };

    let name = unit.name();
    let icon = {
        let unit = unit.clone();
        move || {
            let select = store.select().read();
            let is_selected = select.is_selected(&unit);
            match &select.state {
                SelectedState::Cut(_) if is_selected => {
                    Either::Right(Either::Left(view! { <Icon src="cut" /> }))
                }
                SelectedState::Copy(_) if is_selected => {
                    Either::Right(Either::Right(view! { <Icon src="copy" /> }))
                }
                _ => Either::Left(view! { <UnitIcon unit=unit.clone() is_over_drop_zone/> }),
            }
        }
    };

    view! {
        <button
            on:dblclick=ondblclick
            on:click=onclick
            class="grid grid-cols-1 hover:text-white hover:bg-black justify-items-center"
        >
            {icon}
            <span>{name}</span>
        </button>
    }
}

#[component]
fn UnitIcon(unit: Unit, is_over_drop_zone: Signal<bool>) -> impl IntoView {
    let store = use_context::<Store<GlobalState>>().unwrap();

    let download_link = (unit.kind != UnitKind::Dirctory).then_some(view! {
        <a
            id=unit.name()
            download=unit.name()
            href=format!("/download/{}", unit.path.to_str().unwrap_or_default())
            hidden
        ></a>
    });

    let is_dropable = move || is_over_drop_zone.get() && store.password().get().is_some();
    let size = move || {
        if is_dropable() {
            IconSize::Small
        } else {
            IconSize::default()
        }
    };
    view! {
        <BaseIcon
            src={
                let kind = unit.kind.clone();
                move || kind.to_string()
            }
            active=move || !store.select().read().is_selected(&unit)
            size
        />
        {download_link}
    }
}
