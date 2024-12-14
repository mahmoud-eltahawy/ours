use leptos::prelude::*;

#[component]
pub fn Icon<F>(name: &'static str, active: F) -> impl IntoView
where
    F: Fn() -> bool + 'static + Send,
{
    let path = move || {
        if active() {
            format!("{name}.png")
        } else {
            format!("dark/{name}.png")
        }
    };
    view! {
        <img class="m-1 p-1" src={path} width=65/>
    }
}
