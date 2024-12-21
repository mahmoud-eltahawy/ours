use std::path::PathBuf;

use crate::{
    app::{atoms::Icon, GlobalState, GlobalStateStoreFields, SelectedState},
    Unit,
};

use super::atoms::ActiveIcon;
use leptos::{either::either, html, prelude::*};
use leptos_router::components::A;
use reactive_stores::Store;
use server_fn::codec::{MultipartData, MultipartFormData};
use wasm_bindgen::JsCast;
use web_sys::{Blob, Event, FormData, HtmlInputElement};

#[cfg(feature = "ssr")]
use {
    crate::{ServerContext, UnitKind},
    tokio::{
        fs::{copy, remove_dir_all, remove_file, File},
        io::{AsyncWriteExt, BufWriter},
        process::Command,
    },
};

#[component]
pub fn NavBar() -> impl IntoView {
    let store: Store<GlobalState> = use_context().unwrap();
    view! {
        <nav class="flex flex-wrap place-content-center">
            <Home/>
            <Clear/>
            <Download/>
            {
                move || {
                    either!(
                        store.password().get(),
                        Some(password) => view! {
                            <Upload password={password.clone()}/>
                            <Delete password={password.clone()}/>
                            <Mkdir password={password.clone()}/>
                            <Copy password={password.clone()}/>
                            <Cut password={password.clone()}/>
                            <Paste password={password.clone()}/>
                            <ToMp4 password/>

                        },
                        None =>view! {<Admin/>},
                    )
                }
            }
        </nav>
    }
}

#[component]
fn Home() -> impl IntoView {
    let store: Store<GlobalState> = use_context().unwrap();
    let is_active = move || store.current_path().read().file_name().is_some();

    view! {
        <A href="/" class:disabled={move || !is_active()}>
            <ActiveIcon name="home" active={is_active}/>
        </A>
    }
}

#[component]
fn Clear() -> impl IntoView {
    let store = use_context::<Store<GlobalState>>().unwrap();
    let on_click = move |_| {
        store.select().write().clear();
    };

    let is_active = move || !store.select().read().is_clear();

    view! {
        <button
            disabled={move || !is_active()}
            on:click=on_click
        >
            <ActiveIcon active={is_active} name="clear"/>
        </button>
    }
}

#[component]
fn Admin() -> impl IntoView {
    let store = use_context::<Store<GlobalState>>().unwrap();
    let on_click = move |_| {
        *store.login().write() = true;
    };

    view! {
        <button
            on:click=on_click
        >
            <Icon name="admin"/>
        </button>
    }
}

#[server]
pub async fn mp4_remux(targets: Vec<PathBuf>) -> Result<(), ServerFnError> {
    let context = use_context::<ServerContext>().unwrap();
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
        let _ = remove_file(from).await;
    }
    Ok(())
}

#[component]
fn ToMp4(password: String) -> impl IntoView {
    let store = use_context::<Store<GlobalState>>().unwrap();

    let remux = Action::new(move |input: &Vec<PathBuf>| mp4_remux(input.clone()));
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

    let is_active = move || {
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
        <button
            disabled={move || !is_active()}
            on:click=on_click
        >
            <ActiveIcon active={is_active} name="mp4"/>
        </button>
    }
}

#[component]
fn Mkdir(password: String) -> impl IntoView {
    let store = use_context::<Store<GlobalState>>().unwrap();

    let on_click = move |_| {
        *store.mkdir_state().write() = true;
    };

    let is_active = move || store.select().read().is_clear();

    view! {
        <button
            disabled={move || !is_active()}
            on:click=on_click
        >
            <ActiveIcon active={is_active} name="mkdir"/>
        </button>
    }
}

#[server]
pub async fn rm(bases: Vec<Unit>) -> Result<(), ServerFnError> {
    let context = use_context::<ServerContext>().unwrap();
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
    let remove = Action::new(move |input: &Vec<Unit>| rm(input.clone()));
    let on_click = move |_| {
        remove.dispatch(store.select().get_untracked().units.into_iter().collect());
    };

    Effect::new(move || {
        if !remove.pending().get() {
            store.select().write().clear();
            store.units_refetch_tick().update(|x| *x = !*x);
        }
    });

    let is_active = move || !store.select().read().is_clear();

    view! {
        <button
            disabled={move || !is_active()}
            on:click=on_click
        >
            <ActiveIcon active={is_active} name="delete"/>
        </button>
    }
}

#[server]
pub async fn cp(from: Vec<PathBuf>, to: PathBuf) -> Result<(), ServerFnError> {
    let context = use_context::<ServerContext>().unwrap();
    let to = context.root.join(to);
    for base in from.into_iter().map(|x| context.root.join(x)) {
        copy(&base, to.join(base.file_name().unwrap())).await?;
    }
    Ok(())
}

#[server]
pub async fn cp_cut(from: Vec<PathBuf>, to: PathBuf) -> Result<(), ServerFnError> {
    let context = use_context::<ServerContext>().unwrap();
    let to = context.root.join(to);
    for base in from.into_iter().map(|x| context.root.join(x)) {
        copy(&base, to.join(base.file_name().unwrap())).await?;
        remove_file(base).await?;
    }
    Ok(())
}

#[component]
fn Paste(password: String) -> impl IntoView {
    let store: Store<GlobalState> = use_context().unwrap();
    let copy = Action::new(move |_: &()| {
        cp(
            store.select().read_untracked().as_paths(),
            store.current_path().get_untracked(),
        )
    });
    let cut = Action::new(move |_: &()| {
        cp_cut(
            store.select().read_untracked().as_paths(),
            store.current_path().get_untracked(),
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

    let on_click = move |_| match store.select().read().state {
        SelectedState::Copy => {
            copy.dispatch(());
        }
        SelectedState::Cut => {
            cut.dispatch(());
        }
        SelectedState::None => (),
    };

    let is_active = move || {
        let select = store.select().read();
        !select.is_clear() && !matches!(select.state, SelectedState::None)
    };

    view! {
        <button
            disabled={move || !is_active()}
            on:click=on_click
        >
            <ActiveIcon active={is_active} name="paste"/>
        </button>
    }
}

#[component]
fn Copy(password: String) -> impl IntoView {
    let store: Store<GlobalState> = use_context().unwrap();

    let is_active = move || {
        let select = store.select().read();
        !select.is_clear() && !select.has_dirs()
    };

    let on_click = move |_| {
        store.select().write().copy();
    };

    view! {
        <button
            disabled={move || !is_active()}
            on:click=on_click
        >
            <ActiveIcon active={is_active} name="copy"/>
        </button>
    }
}

#[component]
fn Cut(password: String) -> impl IntoView {
    let store: Store<GlobalState> = use_context().unwrap();

    let is_active = move || {
        let select = store.select().read();
        !select.is_clear() && !select.has_dirs()
    };

    let on_click = move |_| {
        store.select().write().cut();
    };

    view! {
        <button
            disabled={move || !is_active()}
            on:click=on_click
        >
            <ActiveIcon active={is_active} name="cut"/>
        </button>
    }
}

#[component]
fn Download() -> impl IntoView {
    let store: Store<GlobalState> = use_context().unwrap();
    let on_click = move |_| {
        store.select().get_untracked().download_selected();
        store.select().write().clear();
    };

    let is_active = move || {
        let select = store.select().read();
        !select.is_clear() && !select.has_dirs()
    };

    view! {
        <button
            disabled={move || !is_active()}
            on:click=on_click
        >
            <ActiveIcon active={is_active} name="download"/>
        </button>
    }
}

#[server(
     input = MultipartFormData,
 )]
async fn upload(multipart: MultipartData) -> Result<(), ServerFnError> {
    let context = use_context::<ServerContext>().unwrap();

    let mut data = multipart.into_inner().unwrap();

    while let Some(mut field) = data.next_field().await? {
        let path = context.root.join(field.name().unwrap().to_string());
        let mut file = BufWriter::new(File::create(path).await?);
        while let Some(chunk) = field.chunk().await? {
            file.write(&chunk).await?;
            file.flush().await?;
        }
    }

    Ok(())
}

#[component]
fn Upload(password: String) -> impl IntoView {
    let store: Store<GlobalState> = use_context().unwrap();

    let is_active = move || store.select().read().is_clear();

    let upload_action = Action::new_local(|data: &FormData| upload(data.clone().into()));
    let on_change = move |ev: Event| {
        ev.prevent_default();
        let target = ev
            .target()
            .unwrap()
            .unchecked_into::<HtmlInputElement>()
            .files()
            .unwrap();
        let data = FormData::new().unwrap();
        let mut i = 0;
        while let Some(file) = target.item(i) {
            let path = store.current_path().read().join(file.name());
            data.append_with_blob(path.to_str().unwrap(), &Blob::from(file))
                .unwrap();
            i += 1;
        }
        upload_action.dispatch_local(data);
    };
    let input_ref: NodeRef<html::Input> = NodeRef::new();

    let on_click = move |_| {
        input_ref.get().unwrap().click();
    };

    Effect::new(move || {
        if !upload_action.pending().get() {
            store.units_refetch_tick().update(|x| *x = !*x);
        }
    });

    view! {
        <Show
            when={move || !upload_action.pending().get()}
            fallback={move || view!{
                <img class="m-1 p-1" src="load.gif" width=65/>
            }}
            >
            <button
                disabled={move || !is_active()}
                on:click=on_click
            >
                <ActiveIcon active={is_active} name="upload"/>
            </button>
            <input node_ref={input_ref} on:change={on_change} type="file" multiple hidden/>
        </Show>
    }
}
