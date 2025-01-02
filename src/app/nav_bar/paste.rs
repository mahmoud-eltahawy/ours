use crate::app::{nav_bar::LoadableTool, GlobalState, GlobalStateStoreFields, SelectedState};
use leptos::prelude::*;
use reactive_stores::Store;
use std::path::PathBuf;

#[cfg(feature = "ssr")]
use {
    crate::ServerContext,
    tokio::fs::{copy, remove_file},
};

#[server]
async fn cp(from: Vec<PathBuf>, to: PathBuf, password: String) -> Result<(), ServerFnError> {
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
async fn cp_cut(from: Vec<PathBuf>, to: PathBuf, password: String) -> Result<(), ServerFnError> {
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
pub fn Paste(password: String) -> impl IntoView {
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

    let copy_finished = move || !copy.pending().get();
    let cut_finished = move || !cut.pending().get();
    let finished = move || cut_finished() && copy_finished();

    Effect::new(move || {
        if finished() {
            store.select().write().clear();
            store.units_refetch_tick().update(|x| *x = !*x);
        }
    });

    let onclick = move || match store.select().get().state {
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
        <Copy password=password.clone() finished=copy_finished />
        <Cut password=password.clone() finished=cut_finished />
        <LoadableTool active name="paste" onclick finished />
    }
}

#[component]
fn Copy<Finished>(password: String, finished: Finished) -> impl IntoView
where
    Finished: Fn() -> bool + Send + Sync + 'static,
{
    let store: Store<GlobalState> = use_context().unwrap();

    let active = move || {
        let select = store.select().read();
        !select.is_clear() && !select.has_dirs()
    };

    let onclick = move || {
        store.select().write().copy(password.clone());
    };

    view! { <LoadableTool active name="copy" onclick finished /> }
}

#[component]
fn Cut<Finished>(password: String, finished: Finished) -> impl IntoView
where
    Finished: Fn() -> bool + Send + Sync + 'static,
{
    let store: Store<GlobalState> = use_context().unwrap();

    let active = move || {
        let select = store.select().read();
        !select.is_clear() && !select.has_dirs()
    };

    let onclick = move || {
        store.select().write().cut(password.clone());
    };

    view! { <LoadableTool active name="cut" onclick finished /> }
}
