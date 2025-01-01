use std::path::PathBuf;

use crate::{
    app::{atoms::Icon, GlobalState, GlobalStateStoreFields, SelectedState},
    Unit,
};

use super::atoms::ActiveIcon;
use leptos::{either::either, prelude::*, tachys::dom::window};
use leptos_router::{hooks::use_navigate, NavigateOptions};
use reactive_stores::Store;
use upload::Upload;

pub mod upload;

#[cfg(feature = "ssr")]
use {
    crate::{ServerContext, UnitKind},
    tokio::{
        fs::{copy, remove_dir_all, remove_file},
        process::Command,
    },
};

#[component]
pub fn NavBar() -> impl IntoView {
    let store: Store<GlobalState> = use_context().unwrap();
    let more = RwSignal::new(true);
    view! {
        <Show
            when=move || more.get()
            fallback=move || {
                view! {
                    <More more/>
                }
            }
        >
            <nav class="fixed top-0 right-0 z-10 h-screen w-24 bg-white flex flex-wrap place-content-center border-2 border-lime-500 rounded-lg">
                <More more/>
                <Home />
                <Clear />
                <Download />
                {move || {
                    either!(
                        store.password().get(),
                            Some(password) => view! {
                                <AdminRequired password/>
                            },
                            None => view! {<Admin/>},
                    )
                }}
            </nav>
        </Show>
    }
}
#[component]
pub fn More(more: RwSignal<bool>) -> impl IntoView {
    let on_click = move |_| {
        more.update(|x| *x = !*x);
    };
    view! {
        <button
            class="border bg-white fixed top-0 right-0 m-1 p-1 rounded-lg"
            on:click=on_click
         >
            <Icon src="more"/>
        </button>
    }
}

#[component]
pub fn AdminRequired(password: String) -> impl IntoView {
    view! {
        <Upload password={password.clone()}/>
        <Delete password={password.clone()}/>
        <Mkdir password={password.clone()}/>
        <Copy password={password.clone()}/>
        <Cut password={password.clone()}/>
        <Paste/>
        <ToMp4 password/>
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
fn Home() -> impl IntoView {
    let store: Store<GlobalState> = use_context().unwrap();
    let navigate = use_navigate();
    let active = move || store.current_path().read().file_name().is_some();

    let onclick = move || navigate("/", NavigateOptions::default());

    view! {
        <Tool name="home" active onclick/>
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
        <Tool name="clear" active onclick/>
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

    view! {
        <Tool name="download" active onclick/>
    }
}

#[component]
fn Admin() -> impl IntoView {
    let store = use_context::<Store<GlobalState>>().unwrap();
    let onclick = move || {
        *store.login().write() = true;
    };

    view! {
        <Tool name="admin" active=|| true onclick/>
    }
}

#[server]
async fn mp4_remux(targets: Vec<PathBuf>, password: String) -> Result<(), ServerFnError> {
    let context = use_context::<ServerContext>().unwrap();
    if password != context.password {
        return Err(ServerFnError::new("wrong password"));
    };
    for target in targets.into_iter().map(|x| context.root.join(x)) {
        let from = context.root.join(target);
        let mut to = from.clone();
        to.set_extension("mp4");
        let _ = remove_file(to.clone()).await;
        Command::new("ffmpeg")
            .arg("-i")
            .arg(from.clone())
            .arg(to)
            .spawn()?
            .wait()
            .await?;
    }
    Ok(())
}

#[component]
fn ToMp4(password: String) -> impl IntoView {
    let store = use_context::<Store<GlobalState>>().unwrap();

    let remux = Action::new(move |input: &Vec<PathBuf>| mp4_remux(input.clone(), password.clone()));
    let on_click = move |_| {
        let targets = store
            .select()
            .read()
            .units
            .iter()
            .filter(|x| x.path.extension().is_some_and(|x| x != "mp4"))
            .map(|x| x.path.clone())
            .collect::<Vec<_>>();

        remux.dispatch(targets);

        store.select().write().clear();
    };

    let active = move || {
        let select = store.select().read();
        !select.is_clear()
            && select
                .units
                .iter()
                .all(|x| matches!(x.kind, crate::UnitKind::Video))
    };

    Effect::new(move || {
        if !remux.pending().get() {
            store.units_refetch_tick().update(|x| *x = !*x);
        }
    });

    view! {
        <button disabled=move || !active() on:click=on_click>
            <ActiveIcon active name="mp4" />
        </button>
    }
}

#[component]
fn Mkdir(password: String) -> impl IntoView {
    let store = use_context::<Store<GlobalState>>().unwrap();

    let on_click = move |_| {
        *store.mkdir_state().write() = Some(password.clone());
    };

    let active = move || store.select().read().is_clear();

    view! {
        <button disabled=move || !active() on:click=on_click>
            <ActiveIcon active name="mkdir" />
        </button>
    }
}

#[server]
pub async fn rm(bases: Vec<Unit>, password: String) -> Result<(), ServerFnError> {
    let context = use_context::<ServerContext>().unwrap();
    if password != context.password {
        return Err(ServerFnError::new("wrong password"));
    };
    for base in bases.into_iter() {
        let path = context.root.join(base.path);
        match base.kind {
            UnitKind::Dirctory => {
                remove_dir_all(path).await?;
            }
            _ => {
                remove_file(path).await?;
            }
        };
    }

    Ok(())
}

#[component]
fn Delete(password: String) -> impl IntoView {
    let store: Store<GlobalState> = use_context().unwrap();
    let remove = Action::new(move |input: &Vec<Unit>| rm(input.clone(), password.clone()));
    let on_click = move |_| {
        if let Ok(true) = window().confirm_with_message("are you sure you want to delete this") {
            remove.dispatch(store.select().get_untracked().units.into_iter().collect());
        };
    };

    Effect::new(move || {
        if !remove.pending().get() {
            store.select().write().clear();
            store.units_refetch_tick().update(|x| *x = !*x);
        }
    });

    let active = move || !store.select().read().is_clear();

    view! {
        <button disabled=move || !active() on:click=on_click>
            <ActiveIcon active name="delete" />
        </button>
    }
}

#[server]
pub async fn cp(from: Vec<PathBuf>, to: PathBuf, password: String) -> Result<(), ServerFnError> {
    let context = use_context::<ServerContext>().unwrap();
    if password != context.password {
        return Err(ServerFnError::new("wrong password"));
    };
    let to = context.root.join(to);
    for base in from.into_iter().map(|x| context.root.join(x)) {
        copy(&base, to.join(base.file_name().unwrap())).await?;
    }
    Ok(())
}

#[server]
pub async fn cp_cut(
    from: Vec<PathBuf>,
    to: PathBuf,
    password: String,
) -> Result<(), ServerFnError> {
    let context = use_context::<ServerContext>().unwrap();
    if password != context.password {
        return Err(ServerFnError::new("wrong password"));
    };
    let to = context.root.join(to);
    for base in from.into_iter().map(|x| context.root.join(x)) {
        copy(&base, to.join(base.file_name().unwrap())).await?;
        remove_file(base).await?;
    }
    Ok(())
}

#[component]
fn Paste() -> impl IntoView {
    let store: Store<GlobalState> = use_context().unwrap();
    let copy = Action::new({
        move |password: &String| {
            cp(
                store.select().read_untracked().as_paths(),
                store.current_path().get_untracked(),
                password.clone(),
            )
        }
    });
    let cut = Action::new(move |password: &String| {
        cp_cut(
            store.select().read_untracked().as_paths(),
            store.current_path().get_untracked(),
            password.clone(),
        )
    });

    Effect::new(move || {
        if !copy.pending().get() {
            store.select().write().clear();
            store.units_refetch_tick().update(|x| *x = !*x);
        }
    });

    Effect::new(move || {
        if !cut.pending().get() {
            store.select().write().clear();
            store.units_refetch_tick().update(|x| *x = !*x);
        }
    });

    let on_click = move |_| match store.select().get().state {
        SelectedState::Copy(password) => {
            copy.dispatch(password);
        }
        SelectedState::Cut(password) => {
            cut.dispatch(password);
        }
        SelectedState::None => (),
    };

    let active = move || {
        let select = store.select().read();
        !select.is_clear() && !matches!(select.state, SelectedState::None)
    };

    view! {
        <button disabled=move || !active() on:click=on_click>
            <ActiveIcon active name="paste" />
        </button>
    }
}

#[component]
fn Copy(password: String) -> impl IntoView {
    let store: Store<GlobalState> = use_context().unwrap();

    let active = move || {
        let select = store.select().read();
        !select.is_clear() && !select.has_dirs()
    };

    let on_click = move |_| {
        store.select().write().copy(password.clone());
    };

    view! {
        <button disabled=move || !active() on:click=on_click>
            <ActiveIcon active name="copy" />
        </button>
    }
}

#[component]
fn Cut(password: String) -> impl IntoView {
    let store: Store<GlobalState> = use_context().unwrap();

    let active = move || {
        let select = store.select().read();
        !select.is_clear() && !select.has_dirs()
    };

    let on_click = move |_| {
        store.select().write().cut(password.clone());
    };

    view! {
        <button disabled=move || !active() on:click=on_click>
            <ActiveIcon active name="cut" />
        </button>
    }
}
