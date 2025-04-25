use leptos::prelude::*;

#[component]
pub fn ActiveIcon<S, F>(name: S, active: F) -> impl IntoView
where
    S: ToString + Send + Clone + 'static,
    F: Fn() -> bool + 'static + Send,
{
    view! { <BaseIcon src=move || name.clone() active=active size=|| IconSize::default() /> }
}

#[derive(Default)]
pub enum IconSize {
    Small,
    #[default]
    Medium,
}

#[component]
pub fn Icon(src: &'static str) -> impl IntoView {
    view! { <SrcIcon src=move || src /> }
}

#[component]
pub fn SrcIcon<S, FSrc>(src: FSrc) -> impl IntoView
where
    S: ToString,
    FSrc: Fn() -> S + 'static + Send,
{
    view! { <BaseIcon src=src size=|| IconSize::default() active=|| true /> }
}

#[component]
pub fn BaseIcon<S, FSrc, FSize, FActive>(src: FSrc, size: FSize, active: FActive) -> impl IntoView
where
    S: ToString,
    FSrc: Fn() -> S + 'static + Send,
    FSize: Fn() -> IconSize + 'static + Send,
    FActive: Fn() -> bool + 'static + Send,
{
    let src = move || {
        let name = format!("{}.png", src().to_string());
        let name = if active() {
            name
        } else {
            format!("dark/{name}")
        };
        format!("public/{name}")
    };
    let width = move || match size() {
        IconSize::Small => 30,
        IconSize::Medium => 65,
    };
    view! { <img class="m-1 p-1" src=src width=width /> }
}
