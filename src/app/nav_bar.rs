use std::path::PathBuf;

use crate::{app::LsResult, UnitKind};

use super::{atoms::Icon, CurrentPath, Selected};
use leptos::{html, logging::log, prelude::*, task::spawn_local};
use leptos_router::components::A;
use server_fn::codec::{MultipartData, MultipartFormData};
use wasm_bindgen::JsCast;
use web_sys::{Blob, Event, FormData, HtmlInputElement};

#[component]
pub fn NavBar() -> impl IntoView {
    let current_path: CurrentPath = use_context().unwrap();
    let is_active = move || current_path.read().file_name().is_some();
    view! {
        <nav class="flex flex-wrap">
            <A href="/" class:disabled={move || !is_active()}>
                <Icon name="home.png" active={is_active}/>
            </A>
            <Clear/>
            <Download/>
            <Upload/>
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

#[server(
     input = MultipartFormData,
 )]
async fn upload(multipart: MultipartData) -> Result<(), ServerFnError> {
    use crate::ServerContext;
    use tokio::{
        fs::File,
        io::{AsyncWriteExt, BufWriter},
    };
    let context = use_context::<ServerContext>().unwrap();

    let mut data = multipart.into_inner().unwrap();

    while let Some(mut field) = data.next_field().await? {
        let path = context.root.join(field.name().unwrap().to_string());
        let mut file = BufWriter::new(File::create(path).await?);
        while let Some(chunk) = field.chunk().await? {
            file.write(&chunk).await?;
            file.flush().await?;
        }
    }

    Ok(())
}

#[component]
fn Upload() -> impl IntoView {
    let selected = use_context::<Selected>().unwrap();
    let current_path: CurrentPath = use_context().unwrap();

    let is_active = move || selected.read().is_empty();

    let upload_action = Action::new_local(|data: &FormData| upload(data.clone().into()));
    let on_change = move |ev: Event| {
        ev.prevent_default();
        let target = ev
            .target()
            .unwrap()
            .unchecked_into::<HtmlInputElement>()
            .files()
            .unwrap();
        let data = FormData::new().unwrap();
        let mut i = 0;
        while let Some(file) = target.item(i) {
            let path = current_path.get().join(file.name());
            data.append_with_blob(&path.to_str().unwrap(), &Blob::from(file))
                .unwrap();
            i += 1;
        }
        upload_action.dispatch_local(data);
    };
    let input_ref: NodeRef<html::Input> = NodeRef::new();

    let on_click = move |_| {
        input_ref.get().unwrap().click();
    };
    view! {
        <button
            disabled={move || !is_active()}
            on:click=on_click
        >
            <Icon active={is_active} name="upload.png"/>
        </button>
        <input node_ref={input_ref} on:change={on_change} type="file" name="file_to_upload" multiple hidden/>
    }
    // <p>
    //     {move || {
    //         if upload_action.input_local().read().is_none() && upload_action.value().read().is_none()
    //         {
    //             "Upload a file.".to_string()
    //         } else if upload_action.pending().get() {
    //             "Uploading...".to_string()
    //         } else if let Some(Ok(_)) = upload_action.value().get() {
    //             "done".to_string()
    //         } else {
    //             format!("{:?}", upload_action.value().get())
    //         }
    //     }}

    // </p>
}
