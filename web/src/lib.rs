pub const TAILWIND: &str = "/tailwind";
pub const HTMX: &str = "/htmx";
pub const FAVICON: &str = "/favicon.ico";
pub const BOXESIN: &str = "/boxesin";
const BOXESID: &str = "BOXES";

use crate::{
    media::{HiddenPlayer, PLAYER_SECTION},
    navbar::{DownloadNativeApp, NavBar},
    utils::path_as_query,
};
use common::{Unit, UnitKind, assets::IconName};
use leptos::{either::Either, prelude::*};
use std::path::PathBuf;

pub mod media;
mod navbar;
pub mod utils;

pub struct IndexPage {
    same_os: bool,
    pub target_dir: PathBuf,
    pub units: Vec<Unit>,
}

impl IndexPage {
    pub fn new(root: PathBuf, same_os: bool) -> Self {
        Self {
            same_os,
            target_dir: root,
            units: Vec::new(),
        }
    }

    pub fn render(self) -> String {
        let IndexPage {
            units,
            target_dir,
            same_os,
        } = self;

        view! {
        <!DOCTYPE html>
        <html>
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no"/>
                <script src={HTMX}></script>
                <script src={TAILWIND}></script>
                <title>Ours</title>
            </head>
            <body>
                <header>
                    <DownloadNativeApp same_os/>
                </header>
                <Boxes units target_dir parent={PathBuf::new()} is_downloadable={false}/>
                <footer>
                    <HiddenPlayer/>
                </footer>
            </body>
        </html>
        }
        .to_html()
    }
}

impl BoxesProps {
    pub fn to_html(self) -> String {
        Boxes(self).to_html()
    }
}

#[component]
pub fn Boxes(
    units: Vec<Unit>,
    target_dir: PathBuf,
    parent: PathBuf,
    is_downloadable: bool,
) -> impl IntoView {
    let units_view = units
        .into_iter()
        .map(|unit| {
            view! {
                <UnitComp unit=unit target_dir=target_dir.clone() is_downloadable/>
            }
        })
        .collect_view();
    view! {
        <main
            id={BOXESID}
        >
            <NavBar is_downloadable parent/>
            <div
                class="flex flex-wrap w-full min-h-80 m-2 p-2 border-2 border-lime-500 rounded-lg"
            >
                {units_view}
            </div>
        </main>
    }
}

#[component]
fn UnitComp(unit: Unit, target_dir: PathBuf, is_downloadable: bool) -> impl IntoView {
    let name = unit.name();
    let path = unit.path.strip_prefix(target_dir).unwrap().to_path_buf();

    enum Hxs {
        File {
            get: String,
        },
        Other {
            get: String,
            target: String,
            url: String,
        },
    }

    let download_url = format!("/download/{}", path.to_str().unwrap_or_default());

    let hxs = match unit.kind {
        UnitKind::Folder => Hxs::Other {
            get: format!("{}/nah{}", BOXESIN, path_as_query(&path)),
            target: format!("#{}", BOXESID),
            url: utils::path_as_url(&path),
        },
        UnitKind::Video => Hxs::Other {
            get: format!("{}{}", media::VIDEO_HREF, path_as_query(&path)),
            target: format!("#{}", PLAYER_SECTION),
            url: "false".to_string(),
        },
        UnitKind::Audio => Hxs::Other {
            get: format!("{}{}", media::AUDIO_HREF, path_as_query(&path)),
            target: format!("#{}", PLAYER_SECTION),
            url: "false".to_string(),
        },
        UnitKind::File => Hxs::File {
            get: download_url.clone(),
        },
    };

    let download_a = match unit.kind {
        UnitKind::Video | UnitKind::Audio if is_downloadable => Some(view! {
            <a href={download_url} download>
                <Icon name={IconName::Download}/>
            </a>
        }),
        _ => None,
    };

    let children = view! {
        <div>
            <Icon name={IconName::from(unit.kind)} />
            <span>{name.clone()}</span>
        </div>
    };

    let class = "m-5 p-4 grid grid-cols-2 gap-2 justify-items-left hover:text-white hover:bg-black";

    match hxs {
        Hxs::File { get } => Either::Left(view! {
            <a href={get} class={class} download>
                {children}
            </a>
        }),
        Hxs::Other { get, target, url } => Either::Right(view! {
            <div>
                <button
                    hx-get={get}
                    hx-target={target}
                    hx-push-url={url}
                    hx-swap="outerHTML"
                    hx-trigger="pointerup"
                    class={class}
                >
                    {children}
                </button>
                {download_a}
            </div>
        }),
    }
}

#[component]
pub fn Icon(name: IconName) -> impl IntoView {
    let iu: u8 = name.into();
    let src = format!("/icon/{iu}");
    view! {
        <img
            width="40"
            height="40"
            src={src}
        />
    }
}

#[derive(Clone)]
pub struct Context {
    pub target_dir: PathBuf,
}
