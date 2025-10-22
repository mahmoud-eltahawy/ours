use std::{
    env::home_dir,
    ffi::OsStr,
    path::{Path, PathBuf},
};

use crate::{
    BOXESIN, Context, HTMX, TAILWIND,
    components::media::{HiddenPlayer, PLAYER_SECTION},
};
use axum::{
    extract::{self, State},
    response::Html,
};
use common::{AUDIO_X, Unit, UnitKind, VIDEO_X};
use leptos::{either::Either, prelude::*};
use tokio::fs;

pub mod media;

const BOXESID: &str = "BOXES";

pub struct IndexPage {
    target_dir: PathBuf,
    units: Vec<Unit>,
}

impl IndexPage {
    fn new(root: PathBuf) -> Self {
        Self {
            target_dir: root,
            units: Vec::new(),
        }
    }

    async fn fetch_data(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let units = ls(PathBuf::new(), home_dir().unwrap()).await?;
        self.units = units;
        Ok(())
    }

    fn render(self) -> String {
        let IndexPage { units, target_dir } = self;
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
                <Boxes units target_dir parent={PathBuf::new()}/>
                <footer>
                    <HiddenPlayer/>
                </footer>
            </body>
        </html>
        }
        .to_html()
    }

    pub async fn handle(State(Context { target_dir }): State<Context>) -> Html<String> {
        let mut data = Self::new(target_dir);
        data.fetch_data().await.unwrap();
        Html(data.render())
    }
}

pub async fn ls(
    target_dir: PathBuf,
    base: PathBuf,
) -> Result<Vec<Unit>, Box<dyn std::error::Error>> {
    let root = target_dir.join(base);
    let mut dir = fs::read_dir(&root).await?;
    let mut units = Vec::new();
    while let Some(x) = dir.next_entry().await? {
        let kind = if x.file_type().await?.is_dir() {
            UnitKind::Folder
        } else {
            let ex = x.path();
            let ex = ex.extension().and_then(|x| x.to_str());
            match ex {
                Some(ex) => {
                    if VIDEO_X.contains(&ex) {
                        UnitKind::Video
                    } else if AUDIO_X.contains(&ex) {
                        UnitKind::Audio
                    } else {
                        UnitKind::File
                    }
                }
                _ => UnitKind::File,
            }
        };
        let unit = Unit {
            path: x.path().to_path_buf(),
            kind,
        };
        units.push(unit);
    }
    units.sort_by_key(|x| (x.kind.clone(), x.name()));
    Ok(units)
}

const DOWNLAODABLE: &str = "/downloadable";

#[component]
pub fn Boxes(units: Vec<Unit>, target_dir: PathBuf, parent: PathBuf) -> impl IntoView {
    let units_view = units
        .into_iter()
        .map(|unit| {
            view! {
                <UnitComp unit=unit base=target_dir.clone()/>
            }
        })
        .collect_view();
    let d = format!("{}{}", DOWNLAODABLE, path_as_query(&parent));
    view! {
        <main
            id={BOXESID}
        >
            <div class="flex place-content-around m-2 p-2">
                <button
                    hx-get={d}
                    hx-target={format!("#{}",BOXESID)}
                >
                    <Icon name={String::from("download")}/>
                </button>
                <button>
                    <Icon name={String::from("upload")}/>
                </button>
            </div>
            <div
                class="flex flex-wrap w-full min-h-80 m-2 p-2 border-2 border-lime-500 rounded-lg"
            >
                {units_view}
            </div>
        </main>
    }
}

pub async fn boxes_in(
    extract::Query(mut params): extract::Query<Vec<(usize, String)>>,
    State(Context { target_dir }): State<Context>,
) -> Html<String> {
    params.sort_by_key(|x| x.0);
    let parent = params.into_iter().map(|(_, x)| x).collect::<PathBuf>();

    let units = ls(target_dir.clone(), parent.clone()).await.unwrap();

    Html(
        view! {
            <Boxes units target_dir=target_dir parent/>
        }
        .to_html(),
    )
}

fn path_as_query(path: &Path) -> String {
    let mut it = path.iter();
    let kv = |(i, x): (_, &OsStr)| format!("{}={}", i, x.to_str().unwrap());

    let prefix = String::from("?");
    let first = it
        .next()
        .map(|x| prefix.clone() + &kv((0, x)))
        .unwrap_or_default();

    it.enumerate()
        .map(|(i, x)| (i + 1, x))
        .map(kv)
        .fold(first, |acc, x| acc + "&&" + &x)
}

#[component]
fn UnitComp(unit: Unit, base: PathBuf) -> impl IntoView {
    let name = unit.name();
    let path = unit.path.strip_prefix(base).unwrap().to_path_buf();

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

    let hxs = match unit.kind {
        UnitKind::Folder => Hxs::Other {
            get: format!("{}{}", BOXESIN, path_as_query(&path)),
            target: format!("#{}", BOXESID),
            url: path_as_url(&path),
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
            get: format!("/download/{}", path.to_str().unwrap_or_default()),
        },
    };

    let children = view! {
        <Icon name={unit.kind.to_string()} />
        <span>{name.clone()}</span>
    };
    let class = "m-5 p-4 grid grid-cols-2 gap-4 hover:text-white hover:bg-black justify-items-left";

    match hxs {
        Hxs::File { get } => Either::Left(view! {
            <a href={get} class={class} download>
                {children}
            </a>
        }),
        Hxs::Other { get, target, url } => Either::Right(view! {
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
        }),
    }
}

fn path_as_url(path: &Path) -> String {
    path.iter()
        .fold(String::new(), |acc, x| acc + "/" + x.to_str().unwrap())
}

#[component]
pub fn Icon(name: String) -> impl IntoView {
    let src = format!("/icon/{name}");
    view! {
        <img
            width="40"
            height="40"
            src={src}
        />
    }
}
