use super::Context;
use crate::app_error;
use axum::{Json, extract::State};
use std::path::PathBuf;

pub async fn mp4_remux(
    State(Context { root }): State<Context>,
    Json(targets): Json<Vec<PathBuf>>,
) -> Result<(), app_error::AppError> {
    par_mp4_remux(
        targets
            .into_iter()
            .map(|target| root.join(target))
            .collect(),
    )
    .await?;

    Ok(())
}

pub async fn par_mp4_remux(targets: Vec<PathBuf>) -> Result<(), app_error::AppError> {
    use tokio::task::JoinSet;
    let mut set = JoinSet::new();
    targets.into_iter().map(any_to_mp4).for_each(|x| {
        set.spawn(x);
    });

    while let Some(x) = set.join_next().await {
        let Ok(x) = x else {
            return Err(app_error::AppError::FfmpagJoin);
        };
        x?;
    }
    Ok(())
}

pub async fn any_to_mp4(from: PathBuf) -> Result<(), app_error::AppError> {
    use tokio::{fs::remove_file, process::Command};
    let mut to = from.clone();
    to.set_extension("mp4");
    let _ = remove_file(to.clone()).await;
    let Ok(mut child) = Command::new("ffmpeg")
        .arg("-i")
        .arg(from.clone())
        .arg(to)
        .spawn()
    else {
        return Err(app_error::AppError::FfmpagSpawn(from));
    };
    let Ok(_) = child.wait().await else {
        return Err(app_error::AppError::FfmpagWait(from));
    };

    let _ = remove_file(from).await;
    Ok(())
}
