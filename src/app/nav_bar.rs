use std::path::PathBuf;

use crate::{app::LsResult, UnitKind};

use super::{atoms::Icon, CurrentPath, Selected};
use leptos::{logging::log, prelude::*, task::spawn_local};
use leptos_router::components::A;

#[component]
pub fn NavBar(current_path: CurrentPath) -> impl IntoView {
    let is_active = move || current_path.read().file_name().is_some();
    view! {
        <nav class="flex flex-wrap">
            <A href="/" class:disabled={move || !is_active()}>
                <Icon name="home.png" active={is_active}/>
            </A>
            <Clear/>
            <Download/>
            <Delete/>
        </nav>
    }
}

#[component]
fn Clear() -> impl IntoView {
    let selected = use_context::<Selected>().unwrap();
    let on_click = move |_| {
        selected.write().clear();
    };

    let is_active = move || !selected.read().is_empty();

    view! {
        <button
            disabled={move || !is_active()}
            on:click=on_click
        >
            <Icon active={is_active} name="clear.png"/>
        </button>
    }
}

#[server]
pub async fn rm(bases: Vec<PathBuf>) -> Result<(), ServerFnError> {
    use crate::ServerContext;
    use tokio::fs::remove_file;
    let context = use_context::<ServerContext>().unwrap();
    for base in bases.into_iter().map(|x| context.root.join(x)) {
        remove_file(base).await?;
    }

    Ok(())
}

#[component]
fn Delete() -> impl IntoView {
    let selected = use_context::<Selected>().unwrap();
    let ls_result = use_context::<LsResult>().unwrap();
    let on_click = move |_| {
        spawn_local(async move {
            let result = rm(selected
                .read_untracked()
                .iter()
                .map(|x| x.path.clone())
                .collect())
            .await;
            match result {
                Ok(_) => {
                    selected.write().clear();
                    ls_result.refetch();
                }
                Err(e) => log!("Error : {:#?}", e),
            }
        });
    };

    let is_active = move || {
        let list = selected.read();
        !list.is_empty() && !list.iter().any(|x| matches!(x.kind, UnitKind::Dirctory))
    };

    view! {
        <button
            disabled={move || !is_active()}
            on:click=on_click
        >
            <Icon active={is_active} name="delete.png"/>
        </button>
    }
}

#[component]
fn Download() -> impl IntoView {
    let selected = use_context::<Selected>().unwrap();
    let on_click = move |_| {
        for unit in selected.get_untracked().iter() {
            unit.click_anchor();
        }

        selected.write().clear();
    };

    let is_active = move || {
        let list = selected.read();
        !list.is_empty() && !list.iter().any(|x| matches!(x.kind, UnitKind::Dirctory))
    };

    view! {
        <button
            disabled={move || !is_active()}
            on:click=on_click
        >
            <Icon active={is_active} name="download.png"/>
        </button>
    }
}
