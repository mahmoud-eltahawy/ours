use crate::{Unit, UnitKind, Units};
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
use web_sys::KeyboardEvent;

mod atoms;
mod files_box;
mod media_player;
mod nav_bar;

#[cfg(feature = "ssr")]
use crate::ServerContext;

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
#[derive(Default, Clone, Debug)]
enum SelectedState {
    Copy(String),
    Cut(String),
    #[default]
    None,
}

#[derive(Default, Clone, Debug)]
struct Selected {
    units: HashSet<Unit>,
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

    fn copy(&mut self, password: String) {
        self.state = SelectedState::Copy(password);
    }

    fn cut(&mut self, password: String) {
        self.state = SelectedState::Cut(password);
    }

    fn none(&mut self) {
        self.state = SelectedState::None;
    }

    fn remove_unit(&mut self, unit: &Unit) {
        self.units.remove(unit);
        if self.units.is_empty() {
            self.none();
        }
    }

    fn toggle_unit_selection(&mut self, unit: &Unit) {
        if !self.units.insert(unit.clone()) {
            self.remove_unit(unit);
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

#[derive(Clone, Debug, Default, Store)]
struct GlobalState {
    select: Selected,
    current_path: PathBuf,
    media_play: Option<Unit>,
    units: Vec<Unit>,
    units_refetch_tick: bool,
    mkdir_state: Option<String>,
    password: Option<String>,
    login: bool,
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
            *store.units().write() = xs.resort();
        };
    });

    Effect::new(move || {
        let _ = store.units_refetch_tick().read();
        ls_result.refetch();
    });

    window_event_listener(ev::popstate, move |_| {
        store.select().write().clear();
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
            <Login/>
        </Router>
    }
}

#[server]
async fn login(password: String) -> Result<Option<String>, ServerFnError> {
    let context = use_context::<ServerContext>().unwrap();
    let result = if password == context.password {
        Some(password)
    } else {
        None
    };
    Ok(result)
}

#[component]
fn Login() -> impl IntoView {
    let store: Store<GlobalState> = use_context().unwrap();
    let pass = RwSignal::new(String::new());

    let clean = move || {
        *pass.write() = String::new();
        *store.login().write() = false;
    };

    let try_login = Action::new(move |input: &String| login(input.clone()));

    let submit = move || {
        try_login.dispatch(pass.get_untracked());
        clean();
    };

    //that is stupid but works for now
    //it sends the password to the server and get it back if it right
    Effect::new(move || {
        if let Some(Ok(password)) = try_login.value().get() {
            *store.password().write() = password;
        }
    });

    let enter = move |ev: KeyboardEvent| {
        if ev.key() == "Enter" {
            submit();
        }
    };

    view! {
        <Show when={move || store.login().get()}>
            <section class="grid grid-cols-1 gap-5 place-content-center bg-white rounded-lg border-black border-2 p-10 absolute top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2">
                <h2 class="text-center text-3xl">Admin Login</h2>
                <input
                    class="p-5 rounded-lg text-2xl border-2 border-black"
                    type="password"
                    bind:value={pass}
                    on:keypress={enter}
                />
                <div class="flex place-content-center gap-5">
                    <button
                        class="text-center text-2xl border-2 border-black rounded-lg p-5 bg-lime-800 hover:bg-lime-500"
                        on:click={move |_| submit()}
                    >okay</button>
                    <button
                        class="text-center text-2xl border-2 border-black rounded-lg p-5 bg-red-800 hover:bg-red-500"
                        on:click={move |_| clean()}
                    >cancel</button>
                </div>
            </section>
        </Show>
    }
}
