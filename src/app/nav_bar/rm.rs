use crate::app::nav_bar::Tool;
use crate::app::{GlobalState, GlobalStateStoreFields};
use leptos::prelude::*;
use leptos::tachys::dom::window;
use reactive_stores::Store;

use crate::Unit;

#[cfg(feature = "ssr")]
use {
    crate::{ServerContext, UnitKind},
    tokio::fs::{remove_dir_all, remove_file},
};

#[server]
async fn rm(bases: Vec<Unit>, password: String) -> Result<(), ServerFnError> {
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
pub fn Remove(password: String) -> impl IntoView {
    let store: Store<GlobalState> = use_context().unwrap();
    let remove = Action::new(move |input: &Vec<Unit>| rm(input.clone(), password.clone()));
    let onclick = move || {
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
        <Tool active name="delete" onclick/>
    }
}
