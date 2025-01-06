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
    tokio::{fs::remove_file, process::Command, task::JoinSet},
};

use server_fn::codec::Cbor;
#[server(
    input = Cbor,
    output = Cbor
)]
async fn mp4_remux(targets: Vec<PathBuf>, password: String) -> Result<(), ServerFnError> {
    let context = use_context::<ServerContext>().unwrap();
    if password != context.password {
        return Err(ServerFnError::new("wrong password"));
    };

    par_mp4_remux(
        targets
            .into_iter()
            .map(|target| context.root.join(target))
            .collect(),
    )
    .await?;

    Ok(())
}

#[cfg(feature = "ssr")]
pub async fn par_mp4_remux(targets: Vec<PathBuf>) -> Result<(), ServerFnError> {
    let mut set = JoinSet::new();
    targets.into_iter().map(any_to_mp4).for_each(|x| {
        set.spawn(x);
    });

    while let Some(x) = set.join_next().await {
        let _ = x?;
    }
    Ok(())
}

#[cfg(feature = "ssr")]
async fn any_to_mp4(from: PathBuf) -> Result<(), ServerFnError> {
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
    Ok(())
}

#[component]
pub fn ToMp4(password: String) -> impl IntoView {
    let store = use_context::<Store<GlobalState>>().unwrap();

    let remux = Action::new(move |input: &Vec<PathBuf>| mp4_remux(input.clone(), password.clone()));
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

    view! { <LoadableTool active name="mp4" onclick finished /> }
}
