use crate::{Unit, UnitKind};
use files_box::{ls, FilesBox};
use leptos::{ev, prelude::*};
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};
use media_player::MediaPlayer;
use nav_bar::NavBar;
use reactive_stores::Store;
use std::{collections::HashSet, path::PathBuf};

mod atoms;
mod files_box;
mod media_player;
mod nav_bar;

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[derive(Clone, Debug, Default, Store)]
struct GlobalState {
    selected: HashSet<Unit>,
    current_path: PathBuf,
    media_play: Option<Unit>,
    units: Vec<Unit>,
    units_refetch_tick: bool,
}

fn retype_units(units: &mut Vec<Unit>) {
    const VIDEO_X: [&str; 38] = [
        "webm", "mkv", "flv", "vob", "ogv", "ogg", "rrc", "gifv", "mng", "mov", "avi", "qt", "wmv",
        "yuv", "rm", "asf", "amv", "mp4", "m4p", "m4v", "mpg", "mp2", "mpeg", "mpe", "mpv", "m4v",
        "svi", "3gp", "3g2", "mxf", "roq", "nsv", "flv", "f4v", "f4p", "f4a", "f4b", "mod",
    ];

    const AUDIO_X: [&str; 20] = [
        "wav", "mp3", "aiff", "raw", "flac", "alac", "ape", "wv", "tta", "aac", "m4a", "ogg",
        "opus", "wma", "au", "gsm", "amr", "ra", "mmf", "cda",
    ];
    units.iter_mut().for_each(|unit| {
        if unit.kind != UnitKind::File {
            return;
        }
        if let Some(x) = unit.path.extension().and_then(|x| x.to_str()) {
            if VIDEO_X.contains(&x) {
                unit.kind = UnitKind::Video;
            } else if AUDIO_X.contains(&x) {
                unit.kind = UnitKind::Audio;
            }
        };
    });
}

fn sort_units(units: Vec<Unit>) -> Vec<Unit> {
    let (mut directories, mut files, mut videos, mut audios) =
        (Vec::new(), Vec::new(), Vec::new(), Vec::new());

    for unit in units.into_iter() {
        let target = match unit.kind {
            UnitKind::Dirctory => &mut directories,
            UnitKind::Video => &mut videos,
            UnitKind::Audio => &mut audios,
            UnitKind::File => &mut files,
        };
        target.push(unit);
    }

    [&mut directories, &mut videos, &mut audios, &mut files]
        .iter_mut()
        .for_each(|xs| xs.sort_by_key(|x| x.name()));

    directories
        .into_iter()
        .chain(videos)
        .chain(audios)
        .chain(files)
        .collect()
}

#[component]
pub fn App() -> impl IntoView {
    let store = Store::new(GlobalState::default());
    let ls_result = Resource::new(move || store.current_path().get(), ls);

    provide_meta_context();
    provide_context(store);

    Effect::new(move || {
        if let Some(mut xs) = ls_result.get().transpose().ok().flatten() {
            retype_units(&mut xs);
            *store.units().write() = sort_units(xs);
        };
    });

    Effect::new(move || {
        let _ = store.units_refetch_tick().read();
        ls_result.refetch();
    });

    window_event_listener(ev::popstate, move |_| {
        store.selected().write().clear();
    });

    view! {
        <Stylesheet id="leptos" href="/pkg/webls.css"/>
        <Title text="Welcome to Leptos"/>
        <Router>
            <NavBar/>
            <main>
                <Routes fallback=|| "Page not found.">
                    <Route path=StaticSegment("") view=FilesBox/>
                </Routes>
            </main>
            <MediaPlayer/>
        </Router>
    }
}
