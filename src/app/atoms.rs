use leptos::prelude::*;

#[component]
pub fn ActiveIcon<F>(name: &'static str, active: F) -> impl IntoView
where
    F: Fn() -> bool + 'static + Send,
{
    let src = move || {
        let name = format!("{name}.png");
        if active() {
            name
        } else {
            format!("dark/{name}")
        }
    };
    view! { <img class="m-1 p-1" src=src width=65 /> }
}

#[component]
pub fn Icon(name: &'static str) -> impl IntoView {
    let name = format!("{name}.png");
    view! { <img class="m-1 p-1" src=name width=65 /> }
}
