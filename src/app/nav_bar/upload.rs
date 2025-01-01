use crate::app::{nav_bar::Tool, GlobalState, GlobalStateStoreFields};
use leptos::{html, prelude::*};
use reactive_stores::Store;
use server_fn::codec::{MultipartData, MultipartFormData};
use wasm_bindgen::JsCast;
use web_sys::{Blob, Event, FormData, HtmlInputElement};

#[cfg(feature = "ssr")]
use {
    crate::ServerContext,
    std::{path::PathBuf, str::FromStr},
    tokio::{
        fs::File,
        io::{AsyncWriteExt, BufWriter},
    },
};

#[server(
     input = MultipartFormData,
 )]
pub async fn upload(multipart: MultipartData) -> Result<(), ServerFnError> {
    let context = use_context::<ServerContext>().unwrap();

    let mut data = multipart.into_inner().unwrap();

    while let Some(mut field) = data.next_field().await? {
        let name = field.name().unwrap();
        let mut path = PathBuf::from_str(name).unwrap();
        let password = path.file_name().unwrap().to_str().unwrap().to_string();
        path.pop();
        if password != context.password {
            continue;
        }
        let path = context.root.join(path);
        let mut file = BufWriter::new(File::create(path).await?);
        while let Some(chunk) = field.chunk().await? {
            file.write(&chunk).await?;
            file.flush().await?;
        }
    }

    Ok(())
}

#[component]
pub(crate) fn Upload(password: String) -> impl IntoView {
    let store: Store<GlobalState> = use_context().unwrap();
    let active = move || store.select().read().is_clear();

    let upload_action = Action::new_local(|data: &FormData| upload(data.clone().into()));
    let on_change = {
        let password = password.clone();
        move |ev: Event| {
            ev.prevent_default();
            let target = ev
                .target()
                .unwrap()
                .unchecked_into::<HtmlInputElement>()
                .files()
                .unwrap();
            let mut i = 0;
            let current_path = store.current_path().read();
            while let Some(file) = target.item(i) {
                let data = FormData::new().unwrap();
                let path = current_path.join(file.name());
                let path = path.join(password.clone());
                data.append_with_blob(path.to_str().unwrap(), &Blob::from(file))
                    .unwrap();
                upload_action.dispatch_local(data);
                i += 1;
            }
        }
    };
    let input_ref: NodeRef<html::Input> = NodeRef::new();

    let onclick = move || {
        input_ref.get().unwrap().click();
    };

    Effect::new(move || {
        if !upload_action.pending().get() {
            store.units_refetch_tick().update(|x| *x = !*x);
        }
    });

    view! {
        <Show
            when=move || !upload_action.pending().get()
            fallback=move || view! { <img class="m-1 p-1" src="load.gif" width=65 /> }
        >
            <Tool name="upload" active onclick/>
            <input node_ref=input_ref on:change=on_change.clone() type="file" multiple hidden />
        </Show>
    }
}
