use crate::{
    app::{nav_bar::LoadableTool, GlobalState, GlobalStateStoreFields},
    UnitKind,
};
use leptos::prelude::*;
use reactive_stores::Store;
use std::path::PathBuf;

#[cfg(feature = "ssr")]
use {
    crate::ServerContext,
    tokio::{fs::remove_file, process::Command},
};

#[server]
async fn mp4_remux(target: PathBuf, password: String) -> Result<(), ServerFnError> {
    let context = use_context::<ServerContext>().unwrap();
    if password != context.password {
        return Err(ServerFnError::new("wrong password"));
    };
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
    Ok(())
}

#[component]
pub fn ToMp4(password: String) -> impl IntoView {
    let store = use_context::<Store<GlobalState>>().unwrap();

    let remux = Action::new(move |input: &PathBuf| mp4_remux(input.clone(), password.clone()));
    let onclick = move || {
        let targets = store
            .select()
            .read()
            .units
            .iter()
            .filter(|x| x.path.extension().is_some_and(|x| x != "mp4"))
            .map(|x| x.path.clone())
            .collect::<Vec<_>>();
        for target in targets {
            remux.dispatch(target);
        }

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

    view! { <LoadableTool active name="mp4" onclick finished /> }
}
