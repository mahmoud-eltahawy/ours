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
        <nav class="sticky top-0 z-10 bg-white flex flex-wrap place-content-center border-2 border-lime-500 rounded-lg">
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
fn Home() -> impl IntoView {
    let store: Store<GlobalState> = use_context().unwrap();
    let active = move || store.current_path().read().file_name().is_some();

    view! {
        <A href="/" class:disabled=move || !active()>
            <ActiveIcon name="home" active />
        </A>
    }
}

#[component]
fn Clear() -> impl IntoView {
    let store = use_context::<Store<GlobalState>>().unwrap();
    let on_click = move |_| {
        store.select().write().clear();
    };

    let active = move || !store.select().read().is_clear();

    view! {
        <button disabled=move || !active() on:click=on_click>
            <ActiveIcon active name="clear" />
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
        <button on:click=on_click>
            <Icon src="admin" />
        </button>
    }
}

#[server]
pub async fn mp4_remux(targets: Vec<PathBuf>, password: String) -> Result<(), ServerFnError> {
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
        let _ = remove_file(from).await;
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
        remove.dispatch(store.select().get_untracked().units.into_iter().collect());
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

#[component]
fn Download() -> impl IntoView {
    let store: Store<GlobalState> = use_context().unwrap();
    let on_click = move |_| {
        store.select().get_untracked().download_selected();
        store.select().write().clear();
    };

    let active = move || {
        let select = store.select().read();
        !select.is_clear() && !select.has_dirs()
    };

    view! {
        <button disabled=move || !active() on:click=on_click>
            <ActiveIcon active name="download" />
        </button>
    }
}

#[server(
     input = MultipartFormData,
 )]
pub async fn upload(multipart: MultipartData) -> Result<(), ServerFnError> {
    use std::str::FromStr;

    let context = use_context::<ServerContext>().unwrap();

    let mut data = multipart.into_inner().unwrap();

    while let Some(mut field) = data.next_field().await? {
        let name = field.name().unwrap();
        let mut path = PathBuf::from_str(name).unwrap();
        let password = path.file_name().unwrap().to_str().unwrap().to_string();
        path.pop();
        if password != context.password {
            continue;
        }
        let path = context.root.join(path);
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
    let active = move || store.select().read().is_clear();

    let upload_action = Action::new_local(|data: &FormData| upload(data.clone().into()));
    let on_change = {
        let password = password.clone();
        move |ev: Event| {
            ev.prevent_default();
            let target = ev
                .target()
                .unwrap()
                .unchecked_into::<HtmlInputElement>()
                .files()
                .unwrap();
            let mut i = 0;
            let current_path = store.current_path().read();
            while let Some(file) = target.item(i) {
                let data = FormData::new().unwrap();
                let path = current_path.join(file.name());
                let path = path.join(password.clone());
                data.append_with_blob(path.to_str().unwrap(), &Blob::from(file))
                    .unwrap();
                upload_action.dispatch_local(data);
                i += 1;
            }
        }
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
            when=move || !upload_action.pending().get()
            fallback=move || view! { <img class="m-1 p-1" src="load.gif" width=65 /> }
        >
            <button disabled=move || !active() on:click=on_click>
                <ActiveIcon active name="upload" />
            </button>
            <input node_ref=input_ref on:change=on_change.clone() type="file" multiple hidden />
        </Show>
    }
}
