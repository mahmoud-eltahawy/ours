use std::path::PathBuf;

use crate::{files_box::path_as_query, Icon};
use assets::{IconData, CLOSE_SVG, COLLAPSE_SVG, DOWNLOAD_SVG, EXPAND_SVG, HOME_SVG, SELECT_SVG};
use common::{GlobalState, GlobalStateStoreFields, SelectedState, Store};
use leptos::prelude::*;
use leptos_router::{hooks::use_navigate, NavigateOptions};
use mp4::ToMp4;
use paste::Paste;
use rm::Remove;
use upload::Upload;

mod mp4;
mod paste;
mod rm;
pub mod upload;

#[component]
pub fn NavBar(current_path: RwSignal<PathBuf>) -> impl IntoView {
    let more = RwSignal::new(true);
    let hidden = move || {
        if more.get() {
            "display:none"
        } else {
            ""
        }
    };
    let transparent = move || {
        if more.get() {
            "background:transparent;border:none"
        } else {
            ""
        }
    };
    view! {
        <nav
            class="fixed right-0 z-10 h-fit w-fit bg-white grid grid-cols-1 place-content-center border-2 border-lime-500 rounded-lg overflow-scroll"
            style=transparent
        >
            <More more />
            <div class="grid grid-cols-2 place-content-center" style=hidden>
                <Home current_path />
                <Open />
                <Selection />
                <Download />
                <Upload current_path />
                <Remove />
                <Mkdir />
                <Paste current_path />
                <ToMp4 />
            </div>
        </nav>
    }
}

#[component]
pub fn More(more: RwSignal<bool>) -> impl IntoView {
    let on_pointerdown = move |_| {
        more.update(|x| *x = !*x);
    };
    let icon = move || {
        let mut icon = if more.get() { EXPAND_SVG } else { COLLAPSE_SVG };
        icon.fill = Some("black");
        icon
    };

    view! {
        <button
            class="flex bg-white m-1 p-1 rounded-lg place-content-center"
            class:fixed=more
            class:top-0=more
            class:right-0=more
            on:pointerdown=on_pointerdown
        >
            <Icon icon />
        </button>
    }
}

#[component]
fn LoadableTool<Active, Onpointerdown, Finished, Icon>(
    icon: Icon,
    active: Active,
    onpointerdown: Onpointerdown,
    finished: Finished,
) -> impl IntoView
where
    Active: Fn() -> bool + Send + Sync + Clone + Copy + 'static,
    Onpointerdown: Fn() + Send + Sync + Clone + 'static,
    Finished: Fn() -> bool + Send + Sync + Clone + Copy + 'static,
    Icon: Fn() -> IconData + Send + Sync + Clone + Copy + 'static,
{
    view! {
        <Show
            when=finished
            fallback=move || {
                view! {
                    <Tool
                        icon=|| icondata::CgSearchLoading.to_owned()
                        active=|| true
                        onpointerdown=|| {}
                    />
                }
            }
        >
            <Tool icon active onpointerdown=onpointerdown.clone() />
        </Show>
    }
}

#[component]
fn Home(current_path: RwSignal<PathBuf>) -> impl IntoView {
    let store: Store<GlobalState> = use_context().unwrap();
    let navigate = use_navigate();
    let active = move || current_path.read().file_name().is_some();

    let onpointerdown = move || {
        if let SelectedState::None = store.select().get().state {
            store.select().write().clear();
        }
        navigate("/", NavigateOptions::default())
    };

    let icon = || HOME_SVG;

    view! { <Tool icon=icon active onpointerdown /> }
}

#[component]
pub fn Tool<Active, Onpointerdown, Icon>(
    active: Active,
    onpointerdown: Onpointerdown,
    icon: Icon,
) -> impl IntoView
where
    Active: Fn() -> bool + Send + Clone + 'static,
    Onpointerdown: Fn() + Send + 'static,
    Icon: Fn() -> assets::IconData + Send + Clone + 'static,
{
    let style = {
        let active = active.clone();
        move || {
            if active() {
                "border-bottom: solid;border-left: dotted;border-right: dotted;"
            } else {
                "border: hidden;"
            }
        }
    };

    view! {
        <button
            class="m-4 p-2 border-700-lime"
            style=style
            on:pointerdown=move |_| onpointerdown()
            disabled=move || !active()
        >
            <Icon icon />
        </button>
    }
}

#[component]
fn Selection() -> impl IntoView {
    let store = use_context::<Store<GlobalState>>().unwrap();

    let icon = move || {
        if store.select().read().on {
            CLOSE_SVG
        } else {
            SELECT_SVG
        }
    };

    let onpointerdown = move || {
        if store.select().read().on {
            store.select().update(|x| x.clear());
        } else {
            store.select().write().on = true;
        }
    };

    view! { <Tool icon active=|| true onpointerdown /> }
}

#[component]
fn Open() -> impl IntoView {
    let store = use_context::<Store<GlobalState>>().unwrap();
    let navigate = use_navigate();

    let target = move || match &store.select().read().units[..] {
        [target] => match target.kind {
            common::UnitKind::Dirctory => Some(target.clone()),
            _ => None,
        },
        _ => None,
    };
    let onpointerdown = move || {
        if let Some(target) = target() {
            navigate(&path_as_query(&target.path), Default::default());
            store.select().write().clear();
        }
    };

    let active = move || target().is_some();

    view! { <Tool icon=|| icondata::TiFolderOpen.to_owned() active onpointerdown /> }
}

#[component]
fn Download() -> impl IntoView {
    let store: Store<GlobalState> = use_context().unwrap();
    let onpointerdown = move || {
        store.select().get_untracked().wasm_download_selected();
        store.select().write().clear();
    };

    let active = move || {
        let select = store.select().read();
        !select.is_clear() && !select.has_dirs()
    };
    let icon = || DOWNLOAD_SVG;

    view! { <Tool icon=icon active onpointerdown /> }
}

#[component]
fn Mkdir() -> impl IntoView {
    let store = use_context::<Store<GlobalState>>().unwrap();

    let onpointerdown = move || {
        *store.mkdir_state().write() = Some(String::new());
    };

    let active = move || store.select().read().is_clear();

    view! { <Tool icon=|| icondata::AiFolderAddFilled.to_owned() active onpointerdown /> }
}
