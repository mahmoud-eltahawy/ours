use std::{
    env::home_dir,
    ffi::OsStr,
    path::{Path, PathBuf},
};

use axum::{
    extract::{self, State},
    response::Html,
};
use common::{AUDIO_X, Unit, UnitKind, VIDEO_X};
use leptos::prelude::*;
use tokio::fs;

use crate::{BOXESIN, Context, HTMX, TAILWIND};
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
        let IndexPage {
            units,
            target_dir: root,
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
                <Boxes units base=root/>
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
pub fn Boxes(units: Vec<Unit>, base: PathBuf) -> impl IntoView {
    let units_view = units
        .into_iter()
        .map(|unit| {
            view! {
                <UnitComp unit=unit base=base.clone()/>
            }
        })
        .collect_view();
    view! {
        <div
            id={BOXESID}
            class="w-full min-h-80 m-5 p-5 border-2 border-lime-500 rounded-lg"
        >
            {units_view}
        </div>
    }
}

pub async fn boxes_in(
    extract::Query(mut params): extract::Query<Vec<(usize, String)>>,
    State(Context { target_dir }): State<Context>,
) -> Html<String> {
    params.sort_by_key(|x| x.0);
    let path = params.into_iter().map(|(_, x)| x).collect::<PathBuf>();

    let units = ls(target_dir.clone(), path).await.unwrap();

    Html(
        view! {
            <Boxes units base=target_dir/>
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
        .unwrap_or(prefix);

    it.enumerate()
        .map(|(i, x)| (i + 1, x))
        .map(kv)
        .fold(first, |acc, x| acc + "&&" + &x)
}

#[component]
fn UnitComp(unit: Unit, base: PathBuf) -> impl IntoView {
    let name = unit.name();
    let path = unit.path.strip_prefix(base).unwrap().to_path_buf();
    let hx_get = match unit.kind {
        UnitKind::Folder => format!("{}{}", BOXESIN, path_as_query(&path)),
        _ => format!("/download/{}", path.to_str().unwrap_or_default()),
    };

    let id = format!("#{}", BOXESID);

    view! {
        <button
            hx-trigger="pointerdown"
            hx-get={hx_get}
            hx-swap="outerHTML"
            hx-target={id}
            hx-push-url={path_as_url(&path)}
            class="grid grid-cols-2 hover:text-white hover:bg-black justify-items-left"
        >
            <UnitIcon unit=unit />
            <span class="mx-0 px-0 py-5">{name.clone()}</span>
        </button>
    }
}

fn path_as_url(path: &Path) -> String {
    path.iter()
        .fold(String::new(), |acc, x| acc + "/" + x.to_str().unwrap())
}

#[component]
fn UnitIcon(unit: Unit) -> impl IntoView {
    view! {
        <div>
            <Icon name={unit.kind.to_string()}/>
        </div>
    }
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
