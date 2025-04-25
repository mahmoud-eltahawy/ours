use crate::mp4::par_mp4_remux;
use crate::{
    Context,
    app_error::{ServerError, ServerResult},
};
use axum::{
    Json,
    extract::{Multipart, State},
};
use common::{Unit, VIDEO_X};
use std::{
    path::{Path, PathBuf},
    str::FromStr,
};
use tokio::{
    fs::File,
    io::{AsyncWriteExt, BufWriter},
};

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

pub async fn rm(
    State(Context { target_dir }): State<Context>,
    Json(bases): Json<Vec<Unit>>,
) -> ServerResult<()> {
    use {
        common::UnitKind,
        tokio::fs::{remove_dir_all, remove_file},
    };
    for base in bases.into_iter() {
        let path = target_dir.join(base.path);
        match base.kind {
            UnitKind::Dirctory => {
                remove_dir_all(path).await?;
            }
            _ => {
                remove_file(path).await?;
            }
        };
    }

    Ok(())
}

pub async fn upload(
    State(Context { target_dir }): State<Context>,
    multipart: Multipart,
) -> ServerResult<()> {
    let mut data = multipart;
    let mut non_mp4_paths = Vec::new();
    while let Some(mut field) = data.next_field().await? {
        let name = field.name().unwrap();
        let path = PathBuf::from_str(name).unwrap();
        let path = target_dir.join(path);
        let mut file = BufWriter::new(File::create(&path).await?);
        while let Some(chunk) = field.chunk().await? {
            file.write_all(&chunk).await?;
            file.flush().await?;
        }
        if path
            .extension()
            .and_then(|x| x.to_str())
            .is_some_and(|x| VIDEO_X.contains(&x) && x != "mp4")
        {
            non_mp4_paths.push(path);
        };
    }
    par_mp4_remux(non_mp4_paths).await?;

    Ok(())
}
