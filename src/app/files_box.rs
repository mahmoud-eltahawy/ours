use std::path::PathBuf;

use super::atoms::Icon;
use crate::{
    app::{GlobalState, GlobalStateStoreFields},
    Unit, UnitKind,
};
use leptos::{either::Either, prelude::*};
use leptos_router::hooks::{use_navigate, use_query_map};
use reactive_stores::Store;

#[server]
pub async fn ls(base: PathBuf) -> Result<Vec<Unit>, ServerFnError> {
    use crate::{ServerContext, Unit, UnitKind};
    use tokio::fs;
    const VIDEO_X: [&str; 38] = [
        "webm", "mkv", "flv", "vob", "ogv", "ogg", "rrc", "gifv", "mng", "mov", "avi", "qt", "wmv",
        "yuv", "rm", "asf", "amv", "mp4", "m4p", "m4v", "mpg", "mp2", "mpeg", "mpe", "mpv", "m4v",
        "svi", "3gp", "3g2", "mxf", "roq", "nsv", "flv", "f4v", "f4p", "f4a", "f4b", "mod",
    ];

    const AUDIO_X: [&str; 20] = [
        "wav", "mp3", "aiff", "raw", "flac", "alac", "ape", "wv", "tta", "aac", "m4a", "ogg",
        "opus", "wma", "au", "gsm", "amr", "ra", "mmf", "cda",
    ];

    let context: ServerContext = use_context().unwrap();
    let root = context.root.join(base);
    let mut dir = fs::read_dir(&root).await?;
    let mut paths = Vec::new();
    while let Some(x) = dir.next_entry().await? {
        let path = x.path();
        let kind = if x.file_type().await?.is_dir() {
            UnitKind::Dirctory
        } else {
            let ex = path.extension().and_then(|x| x.to_str());
            if ex.is_some_and(|x| VIDEO_X.contains(&x)) {
                UnitKind::Video
            } else if ex.is_some_and(|x| AUDIO_X.contains(&x)) {
                UnitKind::Audio
            } else {
                UnitKind::File
            }
        };
        let unit = Unit {
            path: path.strip_prefix(&context.root)?.to_path_buf(),
            kind,
        };
        paths.push(unit);
    }

    Ok(paths)
}

#[component]
pub fn FilesBox() -> impl IntoView {
    let query = use_query_map();
    let store: Store<GlobalState> = use_context().unwrap();
    Effect::new(move || {
        let queries = query.read();
        let mut i = 0;
        let mut result = PathBuf::new();
        while let Some(x) = queries.get_str(&i.to_string()) {
            result.push(x);
            i += 1;
        }
        store.current_path().set(result);
    });

    view! {
        <section class="flex flex-wrap gap-5 m-5 p-5">
            <For
                each={move || store.units().get()}
                key={|x| x.path.clone()}
                let:unit
            >
                <UnitComp unit={unit}/>
            </For>
        </section>
        <MediaPlayer/>
    }
}
#[component]
fn MediaPlayer() -> impl IntoView {
    let store: Store<GlobalState> = use_context().unwrap();

    move || {
        store.media_play().get().map(|x| match x.1 {
            UnitKind::Video => Either::Left(view! {
                <video width="50%" autoplay controls>
                   <source src={x.0}/>
                </video>
            }),
            UnitKind::Audio => Either::Right(view! {
                <audio autoplay controls>
                   <source src={x.0}/>
                </audio>
            }),
            UnitKind::Dirctory | UnitKind::File => unreachable!(),
        })
    }
}

fn path_as_query(mut path: PathBuf) -> String {
    let mut list = Vec::new();
    while let Some(x) = path.file_name() {
        list.push(x.to_str().unwrap().to_string());
        path.pop();
    }
    let result = list
        .into_iter()
        .rev()
        .enumerate()
        .map(|(i, x)| format!("{i}={x}"))
        .collect::<Vec<_>>()
        .join("&&");
    format!("/?{}", result)
}

#[component]
fn UnitComp(unit: Unit) -> impl IntoView {
    let navigate = use_navigate();
    let store = use_context::<Store<GlobalState>>().unwrap();

    let ondblclick = {
        let unit = unit.clone();
        move |_| {
            store.selected().write().clear();
            match &unit.kind {
                UnitKind::Dirctory => {
                    navigate(&path_as_query(unit.path.clone()), Default::default());
                }
                t @ (UnitKind::Video | UnitKind::Audio) => {
                    *store.media_play().write() = Some((
                        format!("/download/{}", unit.path.to_str().unwrap()),
                        t.clone(),
                    ));
                }
                UnitKind::File => {
                    unit.click_anchor();
                    store.selected().update(|xs| {
                        xs.remove(&unit);
                    });
                }
            }
        }
    };
    let onclick = {
        let unit = unit.clone();
        move |_| {
            store.selected().update(|xs| {
                if !xs.insert(unit.clone()) {
                    xs.remove(&unit);
                };
            })
        }
    };

    let name = unit.name();
    view! {
        <button
            on:dblclick=ondblclick
            on:click=onclick
            class="grid grid-cols-1 hover:text-white hover:bg-black justify-items-center"
        >
            <UnitIcon unit={unit}/>
            <span>{name}</span>
        </button>
    }
}

#[component]
fn UnitIcon(unit: Unit) -> impl IntoView {
    let store = use_context::<Store<GlobalState>>().unwrap();
    let name = match unit.kind {
        UnitKind::Dirctory => "directory.png",
        UnitKind::File => "file.png",
        UnitKind::Video => "video.png",
        UnitKind::Audio => "audio.png",
    };

    let download_link = (unit.kind != UnitKind::Dirctory).then_some(view! {
        <a
            id={unit.name()}
            download={unit.name()}
            href={format!("/download/{}", unit.path.to_str().unwrap())}
            hidden></a>
    });

    view! {
        <Icon name active={move || !store.selected().read().contains(&unit)} />
        {download_link}
    }
}
