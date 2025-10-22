use crate::{
    BOXESID, BOXESIN, Icon,
    utils::{app_name_url, path_as_query},
};
use common::assets::IconName;
use leptos::{either::Either, prelude::*};
use std::path::PathBuf;

#[component]
pub(crate) fn NavBar(parent: PathBuf, is_downloadable: bool) -> impl IntoView {
    view! {
        <div class="flex place-content-around m-2 p-2">
            <DownloadButton is_downloadable parent/>
            <a href="/">
                <Icon name={IconName::Home}/>
            </a>
            <UploadButton/>
        </div>

    }
}

#[component]
pub fn DownloadButton(is_downloadable: bool, parent: PathBuf) -> impl IntoView {
    if is_downloadable {
        Either::Right(view! {
            <button
                hx-get={format!("{}/nah{}", BOXESIN, path_as_query(&parent))}
                hx-target={format!("#{}",BOXESID)}
            >
                <Icon name={IconName::Close}/>
            </button>
        })
    } else {
        Either::Left(view! {
            <button
                hx-get={format!("{}/down{}", BOXESIN, path_as_query(&parent))}
                hx-target={format!("#{}",BOXESID)}
            >
                <Icon name={IconName::Download}/>
            </button>
        })
    }
}

#[component]
pub fn UploadButton() -> impl IntoView {
    view! {
        <button>
            <Icon name={IconName::Upload}/>
        </button>
    }
}

#[component]
pub(crate) fn DownloadNativeApp(same_os: bool) -> impl IntoView {
    if same_os {
        Some(view! {
            <h2
                class="flex flex-wrap place-items-center justify-center"
            >
                <span
                    class="m-2 text-red-700 text-xl text-wrap"
                >this is a fallback app for better experience download native app from</span>
                <a
                    class="m-2 text-lime-700 text-2xl"
                    href={app_name_url()}
                    download
                >here</a>
            </h2>
        })
    } else {
        None
    }
}
