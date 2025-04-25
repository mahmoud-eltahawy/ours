use super::Context;
use crate::app_error::ServerResult;
use axum::{Json, extract::State};
use std::path::PathBuf;

pub async fn mp4_remux(
    State(Context { target_dir }): State<Context>,
    Json(targets): Json<Vec<PathBuf>>,
) -> ServerResult<()> {
    par_mp4_remux(
        targets
            .into_iter()
            .map(|target| target_dir.join(target))
            .collect(),
    )
    .await?;

    Ok(())
}

pub async fn par_mp4_remux(targets: Vec<PathBuf>) -> ServerResult<()> {
    use tokio::task::JoinSet;
    let mut set = JoinSet::new();
    targets.into_iter().map(any_to_mp4).for_each(|x| {
        set.spawn(x);
    });

    while let Some(x) = set.join_next().await {
        x??;
    }
    Ok(())
}

pub async fn any_to_mp4(from: PathBuf) -> ServerResult<()> {
    use tokio::{fs::remove_file, process::Command};
    let mut to = from.clone();
    to.set_extension("mp4");
    let _ = remove_file(to.clone()).await;
    Command::new("ffmpeg")
        .arg("-i")
        .arg(from.clone())
        .arg(to)
        .spawn()?
        .wait()
        .await?;

    let _ = remove_file(from).await;
    Ok(())
}
