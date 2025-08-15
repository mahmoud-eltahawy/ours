use crate::nav_bar::LoadableTool;
use crate::DELIVERY;
use common::Store;
use common::{GlobalState, GlobalStateStoreFields, SelectedState};
use leptos::{ev, prelude::*};
use leptos_use::{use_event_listener, use_window};
use std::path::PathBuf;

#[component]
pub fn Paste(current_path: RwSignal<PathBuf>) -> impl IntoView {
    let store: Store<GlobalState> = use_context().unwrap();
    let copy = Action::new_local({
        move |_: &()| {
            DELIVERY.cp(
                store.select().read_untracked().as_paths(),
                current_path.get_untracked(),
            )
        }
    });
    let cut = Action::new_local(move |_: &()| {
        DELIVERY.mv(
            store.select().read_untracked().as_paths(),
            current_path.get_untracked(),
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

    let onpointerdown = move || match store.select().get().state {
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
            onpointerdown();
        }
    });

    view! {
        <Copy finished=copy_finished />
        <Cut finished=cut_finished />
        <LoadableTool active icon=|| icondata::BiPasteRegular.to_owned() onpointerdown finished />
    }
}

#[component]
fn Copy<Finished>(finished: Finished) -> impl IntoView
where
    Finished: Fn() -> bool + Send + Sync + 'static + Clone + Copy,
{
    let store: Store<GlobalState> = use_context().unwrap();

    let active = move || {
        let select = store.select().read();
        !select.is_clear() && !select.has_dirs()
    };

    let onpointerdown = move || {
        store.select().write().copy();
    };

    let _ = use_event_listener(use_window(), ev::keydown, {
        move |ev| {
            if ev.key().as_str() == "c" && ev.ctrl_key() && active() {
                onpointerdown();
            }
        }
    });

    view! { <LoadableTool icon=|| icondata::AiCopyFilled.to_owned() active onpointerdown finished /> }
}

#[component]
fn Cut<Finished>(finished: Finished) -> impl IntoView
where
    Finished: Fn() -> bool + Send + Sync + 'static + Clone + Copy,
{
    let store: Store<GlobalState> = use_context().unwrap();

    let active = move || {
        let select = store.select().read();
        !select.is_clear() && !select.has_dirs()
    };

    let onpointerdown = {
        move || {
            store.select().write().cut();
        }
    };

    let _ = use_event_listener(use_window(), ev::keydown, {
        move |ev| {
            if ev.key().as_str() == "x" && ev.ctrl_key() && active() {
                onpointerdown();
            }
        }
    });

    view! { <LoadableTool active icon=|| icondata::BiCutRegular.to_owned() onpointerdown finished /> }
}
