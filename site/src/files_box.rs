use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

use crate::{nav_bar::Tool, Icon, Unit, DELIVERY};
use common::{GlobalState, GlobalStateStoreFields, SelectedState};
use common::{Store, UnitKind};
use leptos::{either::Either, ev, html::Ol, prelude::*};
use leptos_router::hooks::{use_navigate, use_query_map};
use leptos_use::{use_event_listener, use_window};
use web_sys::KeyboardEvent;

pub fn origin_with(rel: &str) -> String {
    window()
        .location()
        .origin()
        .map(|x| format!("{x}{rel}"))
        .unwrap()
}

#[component]
pub fn FilesBox(
    drop_zone_el: NodeRef<Ol>,
    is_over_drop_zone: Signal<bool>,
    current_path: RwSignal<PathBuf>,
    units: Memo<Option<Vec<Unit>>>,
) -> impl IntoView {
    let query = use_query_map();
    let store: Store<GlobalState> = use_context().unwrap();
    let navigate = use_navigate();

    Effect::new(move || {
        let queries = query.read();
        let mut i = 0;
        let mut result = PathBuf::new();
        while let Some(x) = queries.get_str(&i.to_string()) {
            result.push(x);
            i += 1;
        }

        current_path.set(result);
    });

    let _ = use_event_listener(use_window(), ev::keydown, move |ev| {
        match ev.key().as_str() {
            "Backspace" => {
                let mut path = current_path.get_untracked();
                if path.pop() {
                    navigate(&path_as_query(&path), Default::default());
                    store.select().write().clear();
                }
            }
            "Enter" => {
                if let Some(Unit {
                    path,
                    kind: UnitKind::Dirctory,
                }) = store.select().get_untracked().units.first()
                {
                    navigate(&path_as_query(path), Default::default());
                    store.select().write().clear();
                };
            }
            _ => (),
        };
    });

    view! {
        <ol
            class="w-full min-h-80 m-5 p-5 border-2 border-lime-500 rounded-lg"
            node_ref=drop_zone_el
        >
            <li>
                <Mkdir current_path/>
            </li>
            {
                move || units.get().map(|xs| {
                    xs.into_iter().map(|x| {view! {
                        <UnitComp unit=x is_over_drop_zone />
                    }}).collect_view()
                })
            }
        </ol>
    }
}

#[component]
fn Mkdir(current_path: RwSignal<PathBuf>) -> impl IntoView {
    let store: Store<GlobalState> = use_context().unwrap();
    let mkdir_state = store.mkdir_state();
    let value = RwSignal::new(String::new());

    let mkdir = Action::new_local(move |input: &PathBuf| DELIVERY.mkdir(input.clone()));
    let enter = move |ev: KeyboardEvent| {
        if ev.key() == "Enter" && mkdir_state.get().is_some() {
            let path = current_path.get_untracked();
            let new_path = path.join(value.get_untracked());
            mkdir.dispatch(new_path);
            *mkdir_state.write() = None;
            value.write().clear();
        }
    };

    Effect::new(move || {
        if !mkdir.pending().get() {
            store.units_refetch_tick().update(|x| *x = !*x);
        }
    });

    let when = move || mkdir_state.get().is_some();
    view! {
        <Show when=when>
            <button>
                <Icon icon=RwSignal::new(icondata::AiFolderFilled.to_owned()) />
                <input
                    class="p-2 border-2 border-black text-2xl"
                    on:keypress=enter
                    type="text"
                    bind:value=value
                />
            </button>
        </Show>
    }
}

fn path_as_query(path: &Path) -> String {
    let mut it = path.iter();
    let kv = |(i, x): (_, &OsStr)| format!("{}={}", i, x.to_str().unwrap());

    let prefix = String::from("/?");
    let first = it
        .next()
        .map(|x| prefix.clone() + &kv((0, x)))
        .unwrap_or(prefix);

    it.enumerate()
        .map(|(i, x)| (i + 1, x))
        .map(kv)
        .fold(first, |acc, x| acc + "&&" + &x)
}

#[test]
fn path_as_query_test() {
    use std::{path::PathBuf, str::FromStr};

    let result = path_as_query(&PathBuf::from_str(".config/helix/config.toml").unwrap());
    assert_eq!(result, "/?0=.config&&1=helix&&2=config.toml")
}

#[component]
fn UnitComp(unit: Unit, is_over_drop_zone: Signal<bool>) -> impl IntoView {
    let navigate = use_navigate();
    let store = use_context::<Store<GlobalState>>().unwrap();

    let ondblclick = {
        let unit = unit.clone();
        move |_| match &unit.kind {
            UnitKind::Dirctory => {
                if matches!(store.select().read().state, SelectedState::None) {
                    store.select().write().clear();
                }
                navigate(&path_as_query(&unit.path), Default::default());
            }
            UnitKind::Video | UnitKind::Audio => {
                *store.media_play().write() = Some(unit.clone());
            }
            UnitKind::File => {
                unit.click_anchor();
                store.select().write().remove_unit(&unit);
            }
        }
    };

    let onclick = {
        let unit = unit.clone();
        move |_| {
            store.select().write().toggle_unit_selection(&unit);
        }
    };

    let name = unit.name();
    let icon = {
        let unit = unit.clone();
        move || {
            let select = store.select().read();
            let is_selected = select.is_selected(&unit);
            match &select.state {
                SelectedState::Cut if is_selected => Either::Right(Either::Left(
                    view! { <Icon icon=RwSignal::new(icondata::BiCutRegular.to_owned())  /> },
                )),
                SelectedState::Copy if is_selected => Either::Right(Either::Right(
                    view! { <Icon  icon=RwSignal::new(icondata::BiCopyRegular.to_owned())/> },
                )),
                _ => Either::Left(view! { <UnitIcon unit=unit.clone() is_over_drop_zone /> }),
            }
        }
    };

    view! {
        <li>
            <button
                on:dblclick=ondblclick
                on:click=onclick
                class="grid grid-cols-2 hover:text-white hover:bg-black justify-items-left"
            >
                {icon}
                <span class="mx-0 px-0 py-5">{name}</span>
            </button>
        </li>
    }
}

#[component]
fn UnitIcon(unit: Unit, is_over_drop_zone: Signal<bool>) -> impl IntoView {
    let store = use_context::<Store<GlobalState>>().unwrap();

    let href = origin_with(&format!(
        "/download/{}",
        unit.path.to_str().unwrap_or_default()
    ));
    let download_link = (unit.kind != UnitKind::Dirctory).then_some(view! {
        <a
            id=unit.name()
            download=unit.name()
            href={href}
            hidden
        ></a>
    });

    let icon_kind = match unit.kind {
        UnitKind::Dirctory => icondata::AiFolderFilled,
        UnitKind::Video => icondata::BiVideoRegular,
        UnitKind::Audio => icondata::AiAudioFilled,
        UnitKind::File => icondata::AiFileFilled,
    };

    let icon = RwSignal::new(icon_kind.to_owned());

    Effect::new(move || {
        if is_over_drop_zone.get() {
            icon.write().width = Some("2em");
            icon.write().height = Some("2em");
        } else {
            icon.write().width = Some("4em");
            icon.write().height = Some("4em");
        }
    });

    view! {
        <Tool icon=icon active=move || !store.select().read().is_selected(&unit) onclick=|| {}/>
        // <BaseIcon
        //     src={
        //         let kind = unit.kind.clone();
        //         move || kind.to_string()
        //     }
        //     active=
        //     size
        // />
        {download_link}
    }
}
