use crate::nav_bar::LoadableTool;
use common::Store;
use common::{GlobalState, GlobalStateStoreFields, SelectedState};
use leptos::{ev, prelude::*};
use leptos_use::{use_event_listener, use_window};
use std::path::PathBuf;

async fn cp(targets: Vec<PathBuf>, to: PathBuf) -> Result<(), String> {
    Ok(())
}

async fn mv(targets: Vec<PathBuf>, to: PathBuf) -> Result<(), String> {
    Ok(())
}

#[component]
pub fn Paste() -> impl IntoView {
    let store: Store<GlobalState> = use_context().unwrap();
    let copy = Action::new({
        move |_: &()| {
            cp(
                store.select().read_untracked().as_paths(),
                store.current_path().get_untracked(),
            )
        }
    });
    let cut = Action::new(move |_: &()| {
        mv(
            store.select().read_untracked().as_paths(),
            store.current_path().get_untracked(),
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
        SelectedState::Copy => {
            copy.dispatch(());
        }
        SelectedState::Cut => {
            cut.dispatch(());
        }
        SelectedState::None => (),
    };

    let active = move || {
        let select = store.select().read();
        !select.is_clear() && !matches!(select.state, SelectedState::None)
    };

    let _ = use_event_listener(use_window(), ev::keydown, move |ev| {
        if ev.key().as_str() == "v" && ev.ctrl_key() && active() {
            onclick();
        }
    });

    view! {
        <Copy finished=copy_finished />
        <Cut finished=cut_finished />
        <LoadableTool active name="paste" onclick finished />
    }
}

#[component]
fn Copy<Finished>(finished: Finished) -> impl IntoView
where
    Finished: Fn() -> bool + Send + Sync + 'static,
{
    let store: Store<GlobalState> = use_context().unwrap();

    let active = move || {
        let select = store.select().read();
        !select.is_clear() && !select.has_dirs()
    };

    let onclick = move || {
        store.select().write().copy();
    };

    let _ = use_event_listener(use_window(), ev::keydown, {
        move |ev| {
            if ev.key().as_str() == "c" && ev.ctrl_key() && active() {
                onclick();
            }
        }
    });

    view! { <LoadableTool active name="copy" onclick finished /> }
}

#[component]
fn Cut<Finished>(finished: Finished) -> impl IntoView
where
    Finished: Fn() -> bool + Send + Sync + 'static,
{
    let store: Store<GlobalState> = use_context().unwrap();

    let active = move || {
        let select = store.select().read();
        !select.is_clear() && !select.has_dirs()
    };

    let onclick = {
        move || {
            store.select().write().cut();
        }
    };

    let _ = use_event_listener(use_window(), ev::keydown, {
        move |ev| {
            if ev.key().as_str() == "x" && ev.ctrl_key() && active() {
                onclick();
            }
        }
    });

    view! { <LoadableTool active name="cut" onclick finished /> }
}
