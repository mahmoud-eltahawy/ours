use crate::files_box::origin_with;
use crate::nav_bar::LoadableTool;
use common::{GlobalState, GlobalStateStoreFields};
use common::{Store, RM_PATH};
use leptos::tachys::dom::window;
use leptos::{ev, prelude::*};
use leptos_use::{use_event_listener, use_window};

use crate::Unit;

async fn rm(bases: Vec<Unit>) -> Result<(), String> {
    reqwest::Client::new()
        .post(origin_with(RM_PATH))
        .json(&bases)
        .send()
        .await
        .map_err(|x| x.to_string())?;
    Ok(())
}

#[component]
pub fn Remove() -> impl IntoView {
    let store: Store<GlobalState> = use_context().unwrap();
    let remove = Action::new_local(move |input: &Vec<Unit>| rm(input.clone()));
    let onclick = move || {
        let units = store.select().get_untracked().units;
        if let Ok(true) = window().confirm_with_message(&format!(
            "are you sure you want to delete {:#?}",
            units.iter().map(|x| x.name()).collect::<Vec<_>>()
        )) {
            remove.dispatch(units.into_iter().collect());
        };
    };

    Effect::new(move || {
        if !remove.pending().get() {
            store.select().write().clear();
            store.units_refetch_tick().update(|x| *x = !*x);
        }
    });

    let active = move || !store.select().read().is_clear();
    let finished = move || !remove.pending().get();

    let _ = use_event_listener(use_window(), ev::keydown, move |ev| {
        if ev.key().as_str() == "Delete" && active() {
            onclick();
        }
    });

    view! { <LoadableTool active icon=icondata::AiDeleteFilled onclick finished /> }
}
