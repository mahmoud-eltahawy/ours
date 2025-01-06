use crate::app::GlobalStateStoreFields;

use super::GlobalState;
use leptos::prelude::*;
use reactive_stores::Store;
use web_sys::KeyboardEvent;

#[cfg(feature = "ssr")]
use crate::ServerContext;

use server_fn::codec::Cbor;
#[server(
    input = Cbor,
    output = Cbor
)]
async fn login(password: String) -> Result<bool, ServerFnError> {
    let context = use_context::<ServerContext>().unwrap();
    Ok(password == context.password)
}

#[component]
pub fn Login() -> impl IntoView {
    let store: Store<GlobalState> = use_context().unwrap();
    let pass = RwSignal::new(String::new());

    let clean = move || {
        *pass.write() = String::new();
        *store.login().write() = false;
    };

    let try_login = Action::new(move |input: &String| login(input.clone()));

    let submit = move || {
        try_login.dispatch(pass.get_untracked());
    };

    Effect::new(move || {
        if let Some(Ok(right)) = try_login.value().get() {
            if right {
                *store.password().write() = Some(pass.get_untracked());
                clean()
            } else {
                *pass.write() = String::new();
            }
        }
    });

    let enter = move |ev: KeyboardEvent| {
        if ev.key() == "Enter" {
            submit();
        }
    };

    view! {
        <Show when=move || store.login().get()>
            <section class="grid grid-cols-1 gap-5 place-content-center bg-white rounded-lg border-black border-2 p-10 fixed top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2 z-24">
                <h2 class="text-center text-3xl">Admin Login</h2>
                <input
                    class="p-5 rounded-lg text-2xl border-2 border-black"
                    type="password"
                    bind:value=pass
                    on:keypress=enter
                />
                <div class="flex place-content-center gap-5">
                    <button
                        class="text-center text-2xl border-2 border-black rounded-lg p-5 bg-lime-800 hover:bg-lime-500"
                        on:click=move |_| submit()
                    >
                        okay
                    </button>
                    <button
                        class="text-center text-2xl border-2 border-black rounded-lg p-5 bg-red-800 hover:bg-red-500"
                        on:click=move |_| clean()
                    >
                        cancel
                    </button>
                </div>
            </section>
        </Show>
    }
}
