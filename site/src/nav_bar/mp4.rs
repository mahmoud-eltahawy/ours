use crate::files_box::origin_with;
use crate::nav_bar::LoadableTool;
use common::{GlobalState, GlobalStateStoreFields, MP4_PATH};
use common::{Store, UnitKind};
use leptos::prelude::*;
use std::path::PathBuf;

async fn mp4_remux(targets: Vec<PathBuf>) -> Result<(), String> {
    reqwest::Client::new()
        .post(origin_with(MP4_PATH))
        .json(&targets)
        .send()
        .await
        .map_err(|x| x.to_string())?;
    Ok(())
}

#[component]
pub fn ToMp4() -> impl IntoView {
    let store = use_context::<Store<GlobalState>>().unwrap();

    let remux = Action::new_local(move |input: &Vec<PathBuf>| mp4_remux(input.clone()));
    let onclick = move || {
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
                .all(|x| matches!(x.kind, UnitKind::Video))
            && select
                .units
                .iter()
                .any(|x| x.path.extension().is_some_and(|x| x != "mp4"))
    };

    let finished = move || !remux.pending().get();

    Effect::new(move || {
        if finished() {
            store.units_refetch_tick().update(|x| *x = !*x);
        }
    });

    view! { <LoadableTool active icon=RwSignal::new(icondata::BsFiletypeMp4.to_owned()) onclick finished /> }
}
