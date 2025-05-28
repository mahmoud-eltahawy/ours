use std::path::PathBuf;

use crate::Icon;
use common::{GlobalState, GlobalStateStoreFields, SelectedState, Store};
use info::Info;
use leptos::{either::either, prelude::*};
use leptos_router::{hooks::use_navigate, NavigateOptions};
use mp4::ToMp4;
use paste::Paste;
use rm::Remove;
use send_wrapper::SendWrapper;
use upload::Upload;

mod info;
mod mp4;
mod paste;
mod rm;
pub mod upload;

#[component]
pub fn NavBar(
    files: Signal<Vec<SendWrapper<web_sys::File>>>,
    current_path: RwSignal<PathBuf>,
) -> impl IntoView {
    let store: Store<GlobalState> = use_context().unwrap();
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
            class="fixed right-0 z-10 h-fit w-fit bg-white grid grid-cols-1  place-content-center border-2 border-lime-500 rounded-lg overflow-scroll"
            style=transparent
        >
            <More more />
            <div class="grid grid-cols-2 place-content-center" style=hidden>
                <Home current_path/>
                <Clear />
                <Download />
                {move || {
                    either!(
                        store.password().get(),
                            true => view! {
                                <AdminRequired files current_path/>
                            },
                            false => view! {<Admin/>},
                    )
                }}
                <Info/>
            </div>
        </nav>
    }
}

#[component]
pub fn More(more: RwSignal<bool>) -> impl IntoView {
    let on_click = move |_| {
        more.update(|x| *x = !*x);
    };
    let icon = RwSignal::new(icondata::BiExpandRegular.to_owned());
    Effect::new(move || {
        if more.get() {
            icon.update(|x| x.fill = Some("green"));
        } else {
            icon.update(|x| x.fill = Some("black"));
        }
    });
    view! {
        <button
            class="flex bg-white m-1 p-1 rounded-lg place-content-center"
            class:fixed=more
            class:top-0=more
            class:right-0=more
            on:click=on_click
        >
            <Icon icon/>
        </button>
    }
}

#[component]
pub fn AdminRequired(
    files: Signal<Vec<SendWrapper<web_sys::File>>>,
    current_path: RwSignal<PathBuf>,
) -> impl IntoView {
    view! {
        <Upload files current_path/>
        <Remove />
        <Mkdir />
        <Paste current_path/>
        <ToMp4  />
    }
}

#[component]
fn LoadableTool<Active, OnClick, Finished>(
    icon: &'static icondata_core::IconData,
    active: Active,
    onclick: OnClick,
    finished: Finished,
) -> impl IntoView
where
    Active: Fn() -> bool + Send + Sync + Clone + Copy + 'static,
    OnClick: Fn() + Send + Sync + Clone + 'static,
    Finished: Fn() -> bool + Send + Sync + Clone + Copy + 'static,
{
    view! {
        <Show
            when=finished
            fallback=move || view! { <Tool icon=icondata::CgSearchLoading active=||true onclick=|| {} /> }
        >
            <Tool icon active onclick=onclick.clone() />
        </Show>
    }
}

#[component]
fn Home(current_path: RwSignal<PathBuf>) -> impl IntoView {
    let store: Store<GlobalState> = use_context().unwrap();
    let navigate = use_navigate();
    let active = move || current_path.read().file_name().is_some();

    let onclick = move || {
        if let SelectedState::None = store.select().get().state {
            store.select().write().clear();
        }
        navigate("/", NavigateOptions::default())
    };

    view! {
        <Tool icon=icondata::BiHomeSmileRegular active onclick/>
    }
}

#[component]
fn Tool<Active, OnClick>(
    active: Active,
    onclick: OnClick,
    icon: &'static icondata_core::IconData,
) -> impl IntoView
where
    Active: Fn() -> bool + Send + Clone + Copy + 'static,
    OnClick: Fn() + Send + 'static,
{
    let icon = RwSignal::new(icon.to_owned());
    Effect::new(move || {
        if active() {
            icon.update(|x| x.fill = Some("green"));
        } else {
            icon.update(|x| x.fill = Some("black"));
        }
    });

    view! {
        <button on:click=move |_| onclick() disabled=move || !active()>
            <Icon icon/>
        </button>
    }
}

#[component]
fn Clear() -> impl IntoView {
    let store = use_context::<Store<GlobalState>>().unwrap();
    let onclick = move || {
        store.select().write().clear();
    };

    let active = move || !store.select().read().is_clear();

    view! {
        <Tool icon=icondata::VsClearAll active onclick/>
    }
}

#[component]
fn Download() -> impl IntoView {
    let store: Store<GlobalState> = use_context().unwrap();
    let onclick = move || {
        store.select().get_untracked().download_selected();
        store.select().write().clear();
    };

    let active = move || {
        let select = store.select().read();
        !select.is_clear() && !select.has_dirs()
    };

    view! { <Tool icon=icondata::BiCloudDownloadRegular active onclick /> }
}

#[component]
fn Admin() -> impl IntoView {
    let store = use_context::<Store<GlobalState>>().unwrap();
    let onclick = move || {
        *store.password().write() = true;
    };

    view! { <Tool icon=icondata::RiAdminUserFacesFill active=|| true onclick /> }
}

#[component]
fn Mkdir() -> impl IntoView {
    let store = use_context::<Store<GlobalState>>().unwrap();

    let onclick = move || {
        *store.mkdir_state().write() = Some(String::new());
    };

    let active = move || store.select().read().is_clear();

    view! { <Tool icon=icondata::AiFolderAddFilled active onclick /> }
}
