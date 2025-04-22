use crate::{Retype, Unit, UnitKind};
use files_box::{FilesBox, ls};
use leptos::{ev, html::Ol, prelude::*};
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    StaticSegment,
    components::{Route, Router, Routes},
};
use leptos_use::{
    UseDropZoneOptions, UseDropZoneReturn, use_drop_zone_with_options, use_event_listener,
    use_window,
};
use media_player::MediaPlayer;
use nav_bar::NavBar;
use reactive_stores::Store;
use std::path::PathBuf;

mod atoms;
mod files_box;
mod media_player;
mod nav_bar;

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>

                <link href="video.css" rel="stylesheet"/>
                <script src="video.js" defer/>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <MetaTags />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}
#[derive(Default, Clone, Debug)]
enum SelectedState {
    Copy,
    Cut,
    #[default]
    None,
}

#[derive(Default, Clone, Debug)]
struct Selected {
    units: Vec<Unit>,
    state: SelectedState,
}

impl Selected {
    fn clear(&mut self) {
        self.units.clear();
        self.none();
    }

    fn as_paths(&self) -> Vec<PathBuf> {
        self.units.iter().map(|x| x.path.clone()).collect()
    }

    fn has_dirs(&self) -> bool {
        self.units
            .iter()
            .any(|x| matches!(x.kind, UnitKind::Dirctory))
    }

    fn is_clear(&self) -> bool {
        self.units.is_empty()
    }

    fn copy(&mut self) {
        self.state = SelectedState::Copy;
    }

    fn cut(&mut self) {
        self.state = SelectedState::Cut;
    }

    fn none(&mut self) {
        self.state = SelectedState::None;
    }

    fn remove_unit(&mut self, unit: &Unit) {
        self.units.retain(|x| x != unit);
        if self.units.is_empty() {
            self.none();
        }
    }

    fn toggle_unit_selection(&mut self, unit: &Unit) {
        if self.units.contains(unit) {
            self.remove_unit(unit);
        } else {
            self.units.push(unit.clone());
            self.units.sort_units();
        }
    }

    fn is_selected(&self, unit: &Unit) -> bool {
        self.units.contains(unit)
    }

    fn download_selected(self) {
        for unit in self.units.into_iter() {
            unit.click_anchor();
        }
    }
}

trait SortUnits {
    fn sort_units(&mut self);
}

impl SortUnits for Vec<Unit> {
    fn sort_units(&mut self) {
        self.sort_by_key(|x| (x.kind.clone(), x.name()));
    }
}

#[derive(Clone, Debug, Default, Store)]
struct GlobalState {
    select: Selected,
    current_path: PathBuf,
    media_play: Option<Unit>,
    units: Vec<Unit>,
    units_refetch_tick: bool,
    mkdir_state: Option<String>,
    password: bool,
}

#[component]
pub fn App() -> impl IntoView {
    let store = Store::new(GlobalState::default());
    let ls_result = Resource::new(move || store.current_path().get(), ls);

    provide_meta_context();
    provide_context(store);

    Effect::new(move || {
        if let Some(mut xs) = ls_result.get().transpose().ok().flatten() {
            xs.retype();
            xs.sort_units();
            *store.units().write() = xs;
        };
    });

    Effect::new(move || {
        let _ = store.units_refetch_tick().read();
        ls_result.refetch();
    });

    let _ = use_event_listener(use_window(), ev::popstate, move |_| {
        if let SelectedState::None = store.select().get().state {
            store.select().write().clear();
        }
    });
    let drop_zone_el = NodeRef::<Ol>::new();

    let UseDropZoneReturn {
        is_over_drop_zone,
        files,
    } = use_drop_zone_with_options(drop_zone_el, UseDropZoneOptions::default());

    view! {
        <Stylesheet id="leptos" href="/pkg/webls.css" />
        <Title text="eltahawy's locker" />
        <Router>
            <NavBar files/>
            <main>
                <Routes fallback=|| "Page not found.">
                    <Route
                        path=StaticSegment("")
                        view=move || view! { <FilesBox drop_zone_el is_over_drop_zone /> }
                    />
                </Routes>
            </main>
            <MediaPlayer />
        </Router>
    }
}
