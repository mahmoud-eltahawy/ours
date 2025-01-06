use crate::app::{atoms::Icon, GlobalState, GlobalStateStoreFields, SelectedState};

use super::atoms::ActiveIcon;
use info::Info;
use leptos::{either::either, prelude::*};
use leptos_router::{hooks::use_navigate, NavigateOptions};
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

#[component]
pub fn NavBar(files: Signal<Vec<SendWrapper<web_sys::File>>>) -> impl IntoView {
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
                <Home />
                <Clear />
                <Download />
                {move || {
                    either!(
                        store.password().get(),
                            Some(password) => view! {
                                <AdminRequired password files/>
                            },
                            None => view! {<Admin/>},
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
    let less = move || more.get();
    view! {
        <button
            class="flex border bg-white m-1 p-1 rounded-lg place-content-center"
            class:fixed=less
            class:top-0=less
            class:right-0=less
            on:click=on_click
        >
            <Icon src="more" />
        </button>
    }
}

#[component]
pub fn AdminRequired(
    password: String,
    files: Signal<Vec<SendWrapper<web_sys::File>>>,
) -> impl IntoView {
    view! {
        <Upload password=password.clone() files />
        <Remove password=password.clone() />
        <Mkdir password=password.clone() />
        <Paste password=password.clone() />
        <ToMp4 password />
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
        *store.login().write() = true;
    };

    view! { <Tool name="admin" active=|| true onclick /> }
}

#[component]
fn Mkdir(password: String) -> impl IntoView {
    let store = use_context::<Store<GlobalState>>().unwrap();

    let onclick = move || {
        *store.mkdir_state().write() = Some(password.clone());
    };

    let active = move || store.select().read().is_clear();

    view! { <Tool name="mkdir" active onclick /> }
}
