use std::path::PathBuf;

use crate::nav_bar::LoadableTool;
use common::{GlobalState, GlobalStateStoreFields, Store, UPLOAD_PATH};
use gloo::net::http::Request;
use leptos::wasm_bindgen::JsCast;
use leptos::{html, prelude::*};
use send_wrapper::SendWrapper;
use web_sys::{Blob, Event, FormData, HtmlInputElement};

async fn upload(form_data: FormData) -> Result<(), String> {
    Request::post(UPLOAD_PATH)
        .body(form_data)
        .map_err(|x| x.to_string())?
        .send()
        .await
        .map_err(|x| x.to_string())?;
    Ok(())
}

#[component]
pub fn Upload(
    files: Signal<Vec<SendWrapper<web_sys::File>>>,
    current_path: RwSignal<PathBuf>,
) -> impl IntoView {
    let store: Store<GlobalState> = use_context().unwrap();
    let upload_action = Action::new_local(|data: &FormData| upload(data.clone()));
    let upload_files = RwSignal::new(Vec::<SendWrapper<web_sys::File>>::new());

    Effect::new(move || {
        let current_path = current_path.read_untracked();
        let data = FormData::new().unwrap();
        for file in upload_files.get() {
            let path = current_path.join(file.name());
            data.append_with_blob(path.to_str().unwrap(), &Blob::from((*file).clone()))
                .unwrap();
        }
        upload_action.dispatch_local(data);
    });

    Effect::new(move || {
        *upload_files.write() = files.get();
    });

    let on_change = move |ev: Event| {
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
            result.push(SendWrapper::new(file));
            i += 1;
        }
        *upload_files.write() = result;
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

    let active = move || store.select().read().is_clear();
    let finished = move || !upload_action.pending().get();
    view! {
        <LoadableTool icon=RwSignal::new(icondata::BiUploadRegular.to_owned()) active onclick finished />
        <input node_ref=input_ref on:change=on_change type="file" multiple hidden />
    }
}
