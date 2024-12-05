use crate::UnitKind;

use super::{atoms::Icon, Selected};
use leptos::{logging::log, prelude::*, tachys::dom::document};
use leptos_router::components::A;
use wasm_bindgen::JsCast;

#[component]
pub fn NavBar() -> impl IntoView {
    view! {
        <nav class="flex flex-wrap">
            <A href="/">
                <Icon name="home.png" active={|| true}/>
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
        log!("begin");
        for unit in selected.read_untracked().iter() {
            document()
                .get_element_by_id(&unit.name())
                .unwrap()
                .unchecked_into::<web_sys::HtmlAnchorElement>()
                .click();
        }
        log!("end"); //FIX : throws error here

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
