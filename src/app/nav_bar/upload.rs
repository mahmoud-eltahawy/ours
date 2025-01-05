use crate::app::{nav_bar::LoadableTool, GlobalState, GlobalStateStoreFields};
use leptos::{html, prelude::*};
use reactive_stores::Store;
use send_wrapper::SendWrapper;
use server_fn::codec::{MultipartData, MultipartFormData};
use wasm_bindgen::JsCast;
use web_sys::{Blob, Event, FormData, HtmlInputElement};

#[cfg(feature = "ssr")]
use {
    super::mp4::par_mp4_remux,
    crate::{ServerContext, VIDEO_X},
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
    let mut non_mp4_paths = Vec::new();
    while let Some(mut field) = data.next_field().await? {
        let name = field.name().unwrap();
        let mut path = PathBuf::from_str(name).unwrap();
        let password = path.file_name().unwrap().to_str().unwrap().to_string();
        path.pop();
        if password != context.password {
            continue;
        }
        let path = context.root.join(path);
        let mut file = BufWriter::new(File::create(&path).await?);
        while let Some(chunk) = field.chunk().await? {
            file.write(&chunk).await?;
            file.flush().await?;
        }
        if path
            .extension()
            .and_then(|x| x.to_str())
            .is_some_and(|x| VIDEO_X.contains(&x) && x != "mp4")
        {
            non_mp4_paths.push(path);
        };
    }
    par_mp4_remux(non_mp4_paths).await?;

    Ok(())
}

#[component]
pub fn Upload(password: String, files: Signal<Vec<SendWrapper<web_sys::File>>>) -> impl IntoView {
    let store: Store<GlobalState> = use_context().unwrap();
    let upload_action = Action::new_local(|data: &FormData| upload(data.clone().into()));
    let upload_files = RwSignal::new(Vec::<SendWrapper<web_sys::File>>::new());

    Effect::new(move || {
        let current_path = store.current_path().read_untracked();
        let data = FormData::new().unwrap();
        for file in upload_files.get() {
            let path = current_path.join(file.name());
            let path = path.join(password.clone());
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
        <LoadableTool name="upload" active onclick finished />
        <input node_ref=input_ref on:change=on_change type="file" multiple hidden />
    }
}
