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
            <ClearButton/>
        </nav>
    }
}

#[component]
fn ClearButton() -> impl IntoView {
    let selected = use_context::<Selected>().unwrap();

    view! {
        <button
            on:click=move |_| {
                selected.update(|xs| xs.clear());
            }
        >
            <Icon active={move || !selected.get().is_empty()} name="clear.png"/>
        </button>
    }
}
