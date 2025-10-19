use leptos::prelude::*;

#[component]
pub fn Index() -> impl IntoView {
    view! {
    <!DOCTYPE html>
    <html>
        <head>
            <meta charset="utf-8" />
            <meta name="viewport" content="width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no"/>
            <script src="/htmx"></script>
            <script src="/tailwind"></script>
            <title>Ours</title>
        </head>
        <body>
            <h1
                class="text-3xl fond-bold underline"
                hx-post="/clicked"
                hx-swap="/outerHTML"
            >hello world</h1>
        </body>
    </html>
    }
}

#[component]
pub fn Clicked() -> impl IntoView {
    view! {
        <h1>hello mahmoud</h1>
    }
}
