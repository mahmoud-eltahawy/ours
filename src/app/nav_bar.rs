use crate::UnitKind;

use super::{atoms::Icon, CurrentPath, Selected};
use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn NavBar(current_path: CurrentPath) -> impl IntoView {
    let is_active = move || current_path.read().file_name().is_some();
    view! {
        <nav class="flex flex-wrap">
            <A href="/" class:disabled={move || !is_active()}>
                <Icon name="home.png" active={is_active}/>
            </A>
            <Clear/>
            <Download/>
            <Delete/>
        </nav>
    }
}

#[component]
fn Clear() -> impl IntoView {
    let selected = use_context::<Selected>().unwrap();
    let on_click = move |_| {
        selected.update(|xs| xs.clear());
    };

    let is_active = move || !selected.read().is_empty();

    view! {
        <button
            disabled={move || !is_active()}
            on:click=on_click
        >
            <Icon active={is_active} name="clear.png"/>
        </button>
    }
}

#[component]
fn Delete() -> impl IntoView {
    let selected = use_context::<Selected>().unwrap();
    let on_click = move |_| {
        selected.update(|xs| xs.clear());
    };

    let is_active = move || !selected.read().is_empty();

    view! {
        <button
            disabled={move || !is_active()}
            on:click=on_click
        >
            <Icon active={is_active} name="delete.png"/>
        </button>
    }
}

#[component]
fn Download() -> impl IntoView {
    let selected = use_context::<Selected>().unwrap();
    let on_click = move |_| {
        for unit in selected.get_untracked().iter() {
            unit.click_anchor();
        }

        selected.write().clear();
    };

    let is_active = move || {
        let list = selected.read();
        !list.is_empty() && !list.iter().any(|x| matches!(x.kind, UnitKind::Dirctory))
    };

    view! {
        <button
            disabled={move || !is_active()}
            on:click=on_click
        >
            <Icon active={is_active} name="download.png"/>
        </button>
    }
}
