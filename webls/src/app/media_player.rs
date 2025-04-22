use common::Store;
use leptos::{html::Div, prelude::*};
use leptos_use::{
    UseDraggableOptions, UseDraggableReturn, core::Position, use_draggable_with_options,
};

use crate::Unit;

use super::{GlobalState, GlobalStateStoreFields};

#[component]
pub fn MediaPlayer() -> impl IntoView {
    let store: Store<GlobalState> = use_context().unwrap();
    let media_play = store.media_play();
    let el = NodeRef::<Div>::new();

    let UseDraggableReturn { y, x, .. } = use_draggable_with_options(
        el,
        UseDraggableOptions::default()
            .initial_value(Position { x: 0.0, y: 40.0 })
            .prevent_default(true),
    );
    let style = move || format!("left: {}px; top: {}px", x.get(), y.get().abs());

    move || {
        media_play.get().map(|unit| {
            view! {
                <div
                    node_ref=el
                    class="fixed touch-none bg-white rounded-lg text-2xl px-4 py-2 border border-gray-400/30 shadow-xs hover:shadow-lg select-none cursor-move z-24"
                    style=style
                >
                    <Bar name=unit.name() />
                    <Player unit />
                </div>
            }
        })
    }
}

#[component]
fn Bar(name: String) -> impl IntoView {
    view! {
        <div class="flow-root">
            <span class="float-left mr-10 truncate">{name}</span>
            <CloseButton />
        </div>
    }
}

#[component]
fn CloseButton() -> impl IntoView {
    let store: Store<GlobalState> = use_context().unwrap();
    let media_play = store.media_play();

    let close = move |_| {
        media_play.set(None);
    };

    view! {
        <button
            on:click=close
            class="bg-white rounded-md p-2 inline-flex items-center justify-center text-gray-400 hover:text-gray-500 hover:bg-gray-100 focus:outline-hidden focus:ring-2 focus:ring-inset focus:ring-indigo-500"
        >
            <CloseIcon />
            <span class="sr-only">Close menu</span>
        </button>
    }
}

#[component]
fn CloseIcon() -> impl IntoView {
    view! {
        <svg
            class="h-6 w-6"
            xmlns="http://www.w3.org/2000/svg"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
            aria-hidden="true"
        >
            <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M6 18L18 6M6 6l12 12"
            />
        </svg>
    }
}

#[component]
fn Player(unit: Unit) -> impl IntoView {
    let src = format!("/download/{}", unit.path.to_str().unwrap());
    view! {
        <video
           id="my-player"
           preload="auto"
           class="rounded-lg cursor-default video-js"
           autoplay
           controls
        >
            <source src=src type="video/mp4"/>
            "Your browser does not support the video tag."
        </video>
    }
}
