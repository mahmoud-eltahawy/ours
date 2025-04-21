use crate::app::{GlobalState, GlobalStateStoreFields, SelectedState, atoms::Icon};

use super::atoms::ActiveIcon;
use info::Info;
use leptos::{either::either, prelude::*};
use leptos_router::{NavigateOptions, hooks::use_navigate};
use mp4::ToMp4;
use paste::Paste;
use reactive_stores::Store;
use rm::Remove;
use send_wrapper::SendWrapper;
use upload::Upload;

mod info;
mod mp4;
mod paste;
mod rm;
pub mod upload;

//TODO : add button to navbar to refresh mounted disks

#[component]
pub fn NavBar(files: Signal<Vec<SendWrapper<web_sys::File>>>) -> impl IntoView {
    let store: Store<GlobalState> = use_context().unwrap();
    let more = RwSignal::new(true);
    let hidden = move || {
        if more.get() { "display:none" } else { "" }
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
                <Home />
                <Clear />
                <Download />
                {move || {
                    either!(
                        store.password().get(),
                            true => view! {
                                <AdminRequired files/>
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
    view! {
        <button
            class="flex bg-white m-1 p-1 rounded-lg place-content-center"
            class:fixed=more
            class:top-0=more
            class:right-0=more
            on:click=on_click
        >
            <Icon src="more" />
        </button>
    }
}

#[component]
pub fn AdminRequired(files: Signal<Vec<SendWrapper<web_sys::File>>>) -> impl IntoView {
    view! {
        <Upload files />
        <Remove />
        <Mkdir />
        <Paste />
        <ToMp4  />
    }
}

#[component]
fn Tool<Name, Active, OnClick>(name: Name, active: Active, onclick: OnClick) -> impl IntoView
where
    Name: ToString + Send + Clone + 'static,
    Active: Fn() -> bool + Send + Clone + Copy + 'static,
    OnClick: Fn() + Send + 'static,
{
    let on_click = move |_| onclick();
    view! {
        <button on:click=on_click disabled=move || !active()>
            <ActiveIcon name active />
        </button>
    }
}

#[component]
fn LoadableTool<Name, Active, OnClick, Finished>(
    name: Name,
    active: Active,
    onclick: OnClick,
    finished: Finished,
) -> impl IntoView
where
    Name: ToString + Send + Sync + Clone + Copy + 'static,
    Active: Fn() -> bool + Send + Sync + Clone + Copy + 'static,
    OnClick: Fn() + Send + Sync + Clone + 'static,
    Finished: Fn() -> bool + Send + Sync + 'static,
{
    view! {
        <Show
            when=finished
            fallback=move || view! { <img class="m-1 p-1" src="load.gif" width=65 /> }
        >
            <Tool name active onclick=onclick.clone() />
        </Show>
    }
}

#[component]
fn Home() -> impl IntoView {
    let store: Store<GlobalState> = use_context().unwrap();
    let navigate = use_navigate();
    let active = move || store.current_path().read().file_name().is_some();

    let onclick = move || {
        if let SelectedState::None = store.select().get().state {
            store.select().write().clear();
        }
        navigate("/", NavigateOptions::default())
    };

    view! { <Tool name="home" active onclick /> }
}

#[component]
fn Clear() -> impl IntoView {
    let store = use_context::<Store<GlobalState>>().unwrap();
    let onclick = move || {
        store.select().write().clear();
    };

    let active = move || !store.select().read().is_clear();

    view! { <Tool name="clear" active onclick /> }
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

    view! { <Tool name="download" active onclick /> }
}

#[component]
fn Admin() -> impl IntoView {
    let store = use_context::<Store<GlobalState>>().unwrap();
    let onclick = move || {
        *store.password().write() = true;
    };

    view! { <Tool name="admin" active=|| true onclick /> }
}

#[component]
fn Mkdir() -> impl IntoView {
    let store = use_context::<Store<GlobalState>>().unwrap();

    let onclick = move || {
        *store.mkdir_state().write() = Some(String::new());
    };

    let active = move || store.select().read().is_clear();

    view! { <Tool name="mkdir" active onclick /> }
}
