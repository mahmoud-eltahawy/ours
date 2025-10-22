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
use common::{AUDIO_X, Unit, UnitKind, VIDEO_X, assets::IconName};
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
                <Boxes units target_dir parent={PathBuf::new()} is_downloadable={false}/>
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
                <UnitComp unit=unit base=target_dir.clone() is_downloadable/>
            }
        })
        .collect_view();
    view! {
        <main
            id={BOXESID}
        >
            <div class="flex place-content-around m-2 p-2">
                <DownloadButton is_downloadable parent/>
                <button>
                    <Icon name={IconName::Upload}/>
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

#[component]
fn DownloadButton(is_downloadable: bool, parent: PathBuf) -> impl IntoView {
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

pub async fn boxes_in(
    extract::Query(mut params): extract::Query<Vec<(usize, String)>>,
    extract::Path(down): extract::Path<String>,
    State(Context { target_dir }): State<Context>,
) -> Html<String> {
    params.sort_by_key(|x| x.0);
    let parent = params.into_iter().map(|(_, x)| x).collect::<PathBuf>();

    let units = ls(target_dir.clone(), parent.clone()).await.unwrap();

    let is_downloadable = down == "down";

    Html(
        view! {
            <Boxes units target_dir=target_dir parent is_downloadable/>
        }
        .to_html(),
    )
}

fn path_as_query(path: &Path) -> String {
    let mut it = path.iter();
    let kv = |(i, x): (_, &OsStr)| format!("{}={}", i, x.to_str().unwrap());

    let first = it
        .next()
        .map(|x| String::from("?") + &kv((0, x)))
        .unwrap_or_default();

    it.enumerate()
        .map(|(i, x)| (i + 1, x))
        .map(kv)
        .fold(first, |acc, x| acc + "&&" + &x)
}

#[component]
fn UnitComp(unit: Unit, base: PathBuf, is_downloadable: bool) -> impl IntoView {
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

    let download_url = format!("/download/{}", path.to_str().unwrap_or_default());

    let hxs = match unit.kind {
        UnitKind::Folder => Hxs::Other {
            get: format!("{}/nah{}", BOXESIN, path_as_query(&path)),
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

fn path_as_url(path: &Path) -> String {
    path.iter()
        .fold(String::new(), |acc, x| acc + "/" + x.to_str().unwrap())
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
