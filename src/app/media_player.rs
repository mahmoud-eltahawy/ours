use leptos::{either::Either, prelude::*};
use reactive_stores::Store;

use crate::UnitKind;

use super::{GlobalState, GlobalStateStoreFields};

#[component]
pub fn MediaPlayer() -> impl IntoView {
    let store: Store<GlobalState> = use_context().unwrap();

    move || {
        store.media_play().get().map(|x| {
            let src = format!("/download/{}", x.0);
            match x.1 {
                UnitKind::Video => Either::Left(view! {
                    <video width="50%" autoplay controls>
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
    }
}
