use crate::{
    Context,
    app_error::{ServerError, ServerResult},
};
use axum::{Json, extract::State};
use std::path::{Path, PathBuf};

pub async fn cp(
    State(Context { target_dir }): State<Context>,
    Json((targets, to)): Json<(Vec<PathBuf>, PathBuf)>,
) -> ServerResult<()> {
    use tokio::{fs::copy, task::JoinSet};
    let to = target_dir.join(to);
    let mut set = JoinSet::new();
    for base in targets.into_iter().map(|x| target_dir.join(x)) {
        let name = path_file_name(&base)?;
        set.spawn(copy(base, to.join(name)));
    }

    while let Some(x) = set.join_next().await {
        x??;
    }

    Ok(())
}

fn path_file_name(base: &Path) -> ServerResult<String> {
    let Some(name) = base
        .file_name()
        .and_then(|x| x.to_str().map(|x| x.to_string()))
    else {
        return Err(ServerError::NonePathFilename);
    };
    Ok(name)
}

pub async fn mv(
    State(Context { target_dir }): State<Context>,
    Json((targets, to)): Json<(Vec<PathBuf>, PathBuf)>,
) -> ServerResult<()> {
    use tokio::task::JoinSet;
    let to = target_dir.join(to);
    let mut set = JoinSet::new();
    for base in targets.into_iter().map(|x| target_dir.join(x)) {
        let name = path_file_name(&base)?;
        set.spawn(cut(base, to.join(name)));
    }

    while let Some(x) = set.join_next().await {
        let _ = x?;
    }
    Ok(())
}

pub async fn cut(from: PathBuf, to: PathBuf) -> ServerResult<()> {
    use tokio::fs::{copy, remove_file};
    copy(&from, to).await?;
    remove_file(from).await?;
    Ok(())
}
