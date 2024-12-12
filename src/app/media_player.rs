use leptos::{either::Either, prelude::*};
use reactive_stores::Store;

use crate::UnitKind;

use super::{GlobalState, GlobalStateStoreFields};

#[component]
pub fn MediaPlayer() -> impl IntoView {
    let store: Store<GlobalState> = use_context().unwrap();
    let media_play = store.media_play();

    let player = move || {
        media_play.get().map(|x| {
            let src = format!("/download/{}", x.path.to_str().unwrap());
            match x.kind {
                UnitKind::Video => Either::Left(view! {
                    <video autoplay controls>
                       <source src={src}/>
                    </video>
                }),
                UnitKind::Audio => Either::Right(view! {
                    <audio autoplay controls>
                       <source src={src}/>
                    </audio>
                }),
                UnitKind::Dirctory | UnitKind::File => unreachable!(),
            }
        })
    };
    let name = move || {
        media_play.get().and_then(|x| {
            x.path
                .file_name()
                .and_then(|x| x.to_str().map(|x| x.to_string()))
        })
    };

    view! {
        <section class="w-[80%] border-2">
            <h2>{name}</h2>
            {player}
        </section>
    }
}
