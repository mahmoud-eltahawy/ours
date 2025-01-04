use std::path::PathBuf;

use crate::{
    app::{
        atoms::{BaseIcon, Icon, IconSize},
        GlobalState, GlobalStateStoreFields, SelectedState,
    },
    Unit, UnitKind,
};
use leptos::{either::Either, ev, html::Ol, logging::log, prelude::*};
use leptos_router::hooks::{use_navigate, use_query_map};
use leptos_use::{use_event_listener, use_window};
use reactive_stores::Store;
use web_sys::KeyboardEvent;

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
pub fn FilesBox(drop_zone_el: NodeRef<Ol>, is_over_drop_zone: Signal<bool>) -> impl IntoView {
    let query = use_query_map();
    let store: Store<GlobalState> = use_context().unwrap();
    let navigate = use_navigate();

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

    let _ = use_event_listener(use_window(), ev::keydown, move |ev| {
        let key = ev.key();
        let ctrl = ev.ctrl_key();
        let alt = ev.alt_key();
        let shift = ev.shift_key();
        if ctrl {
            log!("ctrl");
        };
        if shift {
            log!("shift");
        };
        if alt {
            log!("alt")
        };
        log!("keydown : {}", key);
        match key.as_str() {
            "Backspace" => {
                let mut path = store.current_path().get_untracked();
                if path.pop() {
                    navigate(&path_as_query(path), Default::default());
                }
            }
            "Enter" => {
                match &store
                    .select()
                    .get_untracked()
                    .units
                    .iter()
                    .collect::<Vec<_>>()[..]
                {
                    [Unit {
                        path,
                        kind: UnitKind::Dirctory,
                    }] => {
                        navigate(&path_as_query(path.clone()), Default::default());
                    }
                    [] => (),
                    list => {
                        if let Some(Unit {
                            path,
                            kind: UnitKind::Dirctory,
                        }) = list.first()
                        {
                            store.select().write().clear();
                            navigate(&path_as_query(path.clone()), Default::default());
                        }
                    }
                };
            }
            _ => (),
        };
    });

    view! {
        <ol
            class="w-full min-h-80 m-5 p-5 border-2 border-lime-500 rounded-lg"
            node_ref=drop_zone_el
        >
            <li>
                <Mkdir />
            </li>
            <For each=move || store.units().get() key=|x| x.path.clone() let:unit>
                <UnitComp unit=unit is_over_drop_zone />
            </For>
        </ol>
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
                _ => Either::Left(view! { <UnitIcon unit=unit.clone() is_over_drop_zone /> }),
            }
        }
    };

    view! {
        <li>
            <button
                on:dblclick=ondblclick
                on:click=onclick
                class="grid grid-cols-2 hover:text-white hover:bg-black justify-items-left"
            >
                {icon}
                <span class="mx-0 px-0 py-5">{name}</span>
            </button>
        </li>
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
