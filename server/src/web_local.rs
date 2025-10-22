use axum::{
    extract::{self, Query, State},
    response::Html,
};
use axum_extra::{TypedHeader, headers::UserAgent};
use common::{AUDIO_X, Unit, UnitKind, VIDEO_X};
use std::{env::home_dir, path::PathBuf};
use tokio::fs;
use web::{
    Context, IndexPage,
    media::{AudioPlayerProps, HiddenPlayerProps, VideoPlayerProps},
};

pub async fn boxes_in(
    Query(mut params): Query<Vec<(usize, String)>>,
    extract::Path(down): extract::Path<String>,
    State(Context { target_dir }): State<Context>,
) -> Html<String> {
    params.sort_by_key(|x| x.0);
    let parent = params.into_iter().map(|(_, x)| x).collect::<PathBuf>();

    let units = ls(target_dir.clone(), parent.clone()).await.unwrap();

    let is_downloadable = down == "down";

    Html(
        web::BoxesProps {
            units,
            target_dir,
            parent,
            is_downloadable,
        }
        .to_html(),
    )
}

pub async fn fetch_data(page: &mut IndexPage) -> Result<(), Box<dyn std::error::Error>> {
    let units = ls(PathBuf::new(), home_dir().unwrap()).await?;
    page.units = units;
    Ok(())
}

pub(crate) async fn ls(
    target_dir: PathBuf,
    base: PathBuf,
) -> Result<Vec<Unit>, Box<dyn std::error::Error>> {
    let root = target_dir.join(base);
    let mut dir = fs::read_dir(&root).await?;
    let mut units = Vec::new();
    while let Some(x) = dir.next_entry().await? {
        let kind = if x.file_type().await?.is_dir() {
            UnitKind::Folder
        } else {
            let ex = x.path();
            let ex = ex.extension().and_then(|x| x.to_str());
            match ex {
                Some(ex) => {
                    if VIDEO_X.contains(&ex) {
                        UnitKind::Video
                    } else if AUDIO_X.contains(&ex) {
                        UnitKind::Audio
                    } else {
                        UnitKind::File
                    }
                }
                _ => UnitKind::File,
            }
        };
        let unit = Unit {
            path: x.path().to_path_buf(),
            kind,
        };
        units.push(unit);
    }
    units.sort_by_key(|x| (x.kind.clone(), x.name()));
    Ok(units)
}

pub(crate) fn is_same_os(user_agent: UserAgent) -> bool {
    user_agent
        .as_str()
        .to_lowercase()
        .contains(std::env::consts::OS)
}

pub(crate) async fn index_page(
    TypedHeader(user_agent): TypedHeader<UserAgent>,
    State(Context { target_dir }): State<Context>,
) -> Html<String> {
    let same_os = is_same_os(user_agent);
    let mut data = IndexPage::new(target_dir, same_os);
    fetch_data(&mut data).await.unwrap();
    Html(data.render())
}

pub(crate) async fn videoplayer(
    extract::Query(mut params): extract::Query<Vec<(usize, String)>>,
) -> Html<String> {
    params.sort_by_key(|x| x.0);
    let url = params
        .into_iter()
        .map(|(_, x)| x)
        .fold(String::from("/download"), |acc, x| acc + "/" + &x);

    Html(VideoPlayerProps { url }.to_html())
}

pub async fn close_player() -> Html<String> {
    Html(HiddenPlayerProps {}.to_html())
}

pub(crate) async fn audioplayer(
    extract::Query(mut params): extract::Query<Vec<(usize, String)>>,
) -> Html<String> {
    params.sort_by_key(|x| x.0);
    let url = params
        .into_iter()
        .map(|(_, x)| x)
        .fold(String::from("/download"), |acc, x| acc + "/" + &x);

    Html(AudioPlayerProps { url }.to_html())
}
