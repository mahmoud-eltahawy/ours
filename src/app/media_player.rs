use leptos::{either::Either, prelude::*};
use reactive_stores::Store;

use crate::{Unit, UnitKind};

use super::{GlobalState, GlobalStateStoreFields};

#[component]
pub fn MediaPlayer() -> impl IntoView {
    let store: Store<GlobalState> = use_context().unwrap();
    let media_play = store.media_play();

    let player = move |x: Unit| {
        let src = format!("/download/{}", x.path.to_str().unwrap());
        match x.kind {
            UnitKind::Video => Either::Left(view! {
                <video class="h-full w-full rounded-lg" autoplay controls>
                   <source src={src}/>
                  "Your browser does not support the video tag."
                </video>
            }),
            UnitKind::Audio => Either::Right(view! {
                <audio class="h-full w-full rounded-lg mt-10" autoplay controls>
                   <source src={src}/>
                  "Your browser does not support the audio tag."
                </audio>
            }),
            UnitKind::Dirctory | UnitKind::File => unreachable!(),
        }
    };

    move || {
        media_play.get().map(|x| {
            view! {
                <section class="fixed bottom-2 right-2 bg-white text-3xl w-[60%] rounded-lg">
                    <div class="relative">
                        <span class="absolute left-0 top-0 z-10 mr-10 text-nowrap">{x.name()}</span>
                        <span class="absolute right-0 top-0 z-20">
                            <CloseButton/>
                        </span>
                    </div>
                    {player(x)}
                </section>
            }
        })
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
            on:click={close}
            class="bg-white rounded-md p-2 inline-flex items-center justify-center text-gray-400 hover:text-gray-500 hover:bg-gray-100 focus:outline-none focus:ring-2 focus:ring-inset focus:ring-indigo-500"
        >
          <svg class="h-6 w-6" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor" aria-hidden="true">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
          <span class="sr-only">Close menu</span>
        </button>
    }
}
