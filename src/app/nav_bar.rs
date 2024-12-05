use super::{atoms::Icon, Selected};
use leptos::prelude::*;
use leptos_router::components::A;

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
        selected.update(|xs| xs.clear());
    };
    let is_active = move || !selected.read().is_empty();

    view! {
        <button
            disabled={move || !is_active()}
            on:click=on_click
        >
            <Icon active={is_active} name="download.png"/>
        </button>
    }
}
