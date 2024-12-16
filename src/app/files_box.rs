use std::path::PathBuf;

use super::atoms::ActiveIcon;
use crate::{
    app::{atoms::Icon, GlobalState, GlobalStateStoreFields, SelectedState},
    Unit, UnitKind,
};
use leptos::{either::Either, prelude::*};
use leptos_router::hooks::{use_navigate, use_query_map};
use reactive_stores::Store;
use web_sys::KeyboardEvent;

#[server]
pub async fn ls(base: PathBuf) -> Result<Vec<Unit>, ServerFnError> {
    use crate::{ServerContext, Unit, UnitKind};
    use tokio::fs;

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

    view! {
        <section class="flex flex-wrap gap-5 m-5 p-5">
            <Mkdir/>
            <For
                each={move || store.units().get()}
                key={|x| x.path.clone()}
                let:unit
            >
                <UnitComp unit={unit}/>
            </For>
        </section>
    }
}

#[server]
pub async fn mkdir(target: PathBuf) -> Result<(), ServerFnError> {
    use crate::ServerContext;
    use tokio::fs;
    let context = use_context::<ServerContext>().unwrap();
    let target = context.root.join(target);
    fs::create_dir(target).await?;
    Ok(())
}

#[component]
fn Mkdir() -> impl IntoView {
    let store: Store<GlobalState> = use_context().unwrap();
    let mkdir_state = store.mkdir_state();
    let value = RwSignal::new(String::new());

    let mkdir = Action::new(move |input: &PathBuf| mkdir(input.clone()));
    let enter = move |ev: KeyboardEvent| {
        if ev.key() == "Enter" {
            let path = store.current_path().get_untracked();
            let new_path = path.join(value.get_untracked());
            mkdir.dispatch(new_path);
            *mkdir_state.write() = false;
            value.write().clear();
        }
    };

    Effect::new(move || {
        if !mkdir.pending().get() {
            store.units_refetch_tick().update(|x| *x = !*x);
        }
    });

    let when = move || mkdir_state.get();
    view! {
        <Show when={when}>
            <button>
                <Icon name="directory"/>
                <input
                    class="p-2 border-2 border-black text-2xl"
                    on:keypress={enter}
                    type="text"
                    bind:value={value}
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
fn UnitComp(unit: Unit) -> impl IntoView {
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
            match select.state {
                SelectedState::Cut if is_selected => Either::Right(Either::Left(view! {
                    <Icon name="cut"/>
                })),
                SelectedState::Copy if is_selected => Either::Right(Either::Right(view! {
                    <Icon name="copy"/>
                })),
                _ => Either::Left(view! {
                    <UnitIcon unit={unit.clone()}/>
                }),
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
fn UnitIcon(unit: Unit) -> impl IntoView {
    let store = use_context::<Store<GlobalState>>().unwrap();
    let name = match unit.kind {
        UnitKind::Dirctory => "directory",
        UnitKind::File => "file",
        UnitKind::Video => "video",
        UnitKind::Audio => "audio",
    };

    let download_link = (unit.kind != UnitKind::Dirctory).then_some(view! {
        <a
            id={unit.name()}
            download={unit.name()}
            href={format!("/download/{}", unit.path.to_str().unwrap_or_default())}
            hidden></a>
    });

    view! {
        <ActiveIcon name active={move || !store.select().read().is_selected(&unit)} />
        {download_link}
    }
}
