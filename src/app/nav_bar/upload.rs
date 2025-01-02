use crate::app::{nav_bar::LoadableTool, GlobalState, GlobalStateStoreFields};
use leptos::{html, prelude::*};
use leptos_use::UseDropZoneReturn;
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
async fn upload(multipart: MultipartData) -> Result<(), ServerFnError> {
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
pub fn Upload(use_drop_zone_return: UseDropZoneReturn) -> impl IntoView {
    let store: Store<GlobalState> = use_context().unwrap();
    let upload_action = Action::new_local(|data: &FormData| upload(data.clone().into()));
    let upload_files = RwSignal::new(Vec::<send_wrapper::SendWrapper<web_sys::File>>::new());

    Effect::new(move || {
        let current_path = store.current_path().read_untracked();
        let new_files = upload_files.get();
        if let Some(password) = store.password().get_untracked() {
            for file in new_files {
                let data = FormData::new().unwrap();
                let path = current_path.join(file.name());
                let path = path.join(password.clone());
                data.append_with_blob(path.to_str().unwrap(), &Blob::from((*file).clone()))
                    .unwrap();
                upload_action.dispatch_local(data);
            }
        };
    });

    Effect::new(move || {
        *upload_files.write() = use_drop_zone_return.files.get();
    });

    Effect::new(move || {
        if use_drop_zone_return.is_over_drop_zone.get() && store.password().get().is_none() {
            *store.login().write() = true;
        }
    });

    let on_change = {
        move |ev: Event| {
            ev.prevent_default();
            let target = ev
                .target()
                .unwrap()
                .unchecked_into::<HtmlInputElement>()
                .files()
                .unwrap();
            let mut i = 0;
            let mut result = Vec::new();
            while let Some(file) = target.item(i) {
                result.push(send_wrapper::SendWrapper::new(file));
                i += 1;
            }
            *upload_files.write() = result;
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

    let active = move || store.select().read().is_clear() && store.password().get().is_some();
    let finished = move || !upload_action.pending().get();
    view! {
        <LoadableTool name="upload" active onclick finished />
        <input node_ref=input_ref on:change=on_change type="file" multiple hidden />
    }
}
