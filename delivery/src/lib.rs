use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use async_recursion::async_recursion;
use common::{
    AUDIO_X, CP_PATH, LS_PATH, MKDIR_PATH, MP4_PATH, MV_PATH, RM_PATH, Unit, UnitKind, VIDEO_X,
};
use gloo::net::http::Request;
use reqwest::get;
use tokio::io::AsyncWriteExt;

#[derive(Debug, Clone)]
pub struct Delivery {
    origin: Arc<str>,
}

impl Delivery {
    pub fn new(origin: String) -> Self {
        Self {
            origin: Arc::from(origin),
        }
    }

    fn url_path(self, path: &str) -> String {
        format!("{}{}", self.origin, path)
    }

    pub async fn mp4_remux(self, targets: Vec<PathBuf>) -> Result<(), String> {
        reqwest::Client::new()
            .post(self.url_path(MP4_PATH))
            .json(&targets)
            .send()
            .await
            .map_err(|x| x.to_string())?;
        Ok(())
    }

    pub async fn cp(self, targets: Vec<PathBuf>, to: PathBuf) -> Result<(), String> {
        reqwest::Client::new()
            .post(self.url_path(CP_PATH))
            .json(&(targets, to))
            .send()
            .await
            .map_err(|x| x.to_string())?;
        Ok(())
    }

    pub async fn mv(self, targets: Vec<PathBuf>, to: PathBuf) -> Result<(), String> {
        reqwest::Client::new()
            .post(self.url_path(MV_PATH))
            .json(&(targets, to))
            .send()
            .await
            .map_err(|x| x.to_string())?;
        Ok(())
    }

    pub async fn ls(self, base: PathBuf) -> Result<Vec<Unit>, String> {
        let res = reqwest::Client::new()
            .post(self.url_path(LS_PATH))
            .json(&base)
            .send()
            .await
            .map_err(|x| x.to_string())?
            .json::<Vec<Unit>>()
            .await
            .map(retype)
            .map(|mut xs| {
                xs.sort_by_key(|x| (x.kind.clone(), x.name()));
                xs
            })
            .map_err(|x| x.to_string())?;
        Ok(res)
    }

    pub async fn mkdir(self, target: PathBuf) -> Result<(), String> {
        let _ = reqwest::Client::new()
            .post(self.url_path(MKDIR_PATH))
            .json(&target)
            .send()
            .await
            .map_err(|x| x.to_string())?;
        Ok(())
    }

    pub async fn rm(self, bases: Vec<Unit>) -> Result<(), String> {
        reqwest::Client::new()
            .post(self.url_path(RM_PATH))
            .json(&bases)
            .send()
            .await
            .map_err(|x| x.to_string())?;
        Ok(())
    }

    //FIX : should be available on wasm target only
    pub async fn wasm_upload(&self, form_data: web_sys::FormData) -> Result<(), String> {
        Request::post(common::UPLOAD_PATH)
            .body(form_data)
            .map_err(|x| x.to_string())?
            .send()
            .await
            .map_err(|x| x.to_string())?;
        Ok(())
    }

    //FIX : should be available on non wasm target only
    pub async fn iced_upload(&self, _files: &[std::fs::File]) -> Result<(), String> {
        todo!()
    }

    pub async fn download(self, units: Vec<Unit>, pwd: PathBuf) -> Result<(), String> {
        for unit in units.iter() {
            match unit.kind {
                UnitKind::Dirctory => {
                    self.clone().download_directory(unit, &pwd).await?;
                }
                _ => {
                    download_file(self.origin.clone(), unit, &pwd).await?;
                }
            }
        }
        Ok(())
    }

    #[async_recursion]
    async fn download_directory(&self, unit: &Unit, pwd: &Path) -> Result<(), String> {
        let new_units = self.clone().ls(unit.path.clone()).await?;
        let pwd = pwd.join(unit.name());
        tokio::fs::create_dir(&pwd)
            .await
            .map_err(|x| x.to_string())?;
        for unit in new_units.iter() {
            match unit.kind {
                UnitKind::Dirctory => self.clone().download_directory(unit, &pwd).await?,
                _ => download_file(self.origin.clone(), unit, &pwd).await?,
            }
        }
        Ok(())
    }
}

async fn download_file(origin: Arc<str>, unit: &Unit, pwd: &Path) -> Result<(), String> {
    let url = format!(
        "{}/download/{}",
        origin,
        unit.path.to_str().unwrap_or_default()
    );
    let file_path = pwd.join(unit.name());
    let response = get(url).await.map_err(|x| x.to_string())?;
    let mut file = tokio::fs::File::create(file_path)
        .await
        .map_err(|x| x.to_string())?;
    let content = response.bytes().await.map_err(|x| x.to_string())?;
    file.write_all(&content).await.map_err(|x| x.to_string())?;
    Ok(())
}

fn retype(xs: Vec<Unit>) -> Vec<Unit> {
    xs.into_iter()
        .map(|x| match x.kind {
            common::UnitKind::File => {
                let Unit { path, kind } = x;
                let kind = match path.extension().and_then(|x| x.to_str()) {
                    Some(x) => {
                        if VIDEO_X.contains(&x) {
                            UnitKind::Video
                        } else if AUDIO_X.contains(&x) {
                            UnitKind::Audio
                        } else {
                            kind
                        }
                    }
                    None => kind,
                };

                Unit { path, kind }
            }
            _ => x,
        })
        .collect::<Vec<_>>()
}
