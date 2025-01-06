use leptos::prelude::*;

use crate::app::nav_bar::Tool;
#[component]
pub(crate) fn Info() -> impl IntoView {
    // let store = use_context::<Store<GlobalState>>().unwrap();
    let display = RwSignal::new(false);

    let onclick = move || {
        display.update(|x| *x = !*x);
    };

    let active = move || true;
    view! {
       <Tool name="info" active onclick />
       <Show when=move || display.get()>
           <InfoCard/>
       </Show>
    }
}

#[component]
pub(crate) fn InfoCard() -> impl IntoView {
    view! {
        <div class="fixed top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2">
            <h2>hello info</h2>
        </div>
    }
}
