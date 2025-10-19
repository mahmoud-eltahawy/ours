use std::{path::PathBuf, sync::Arc};

use common::{
    AUDIO_X, CP_PATH, LS_PATH, MKDIR_PATH, MP4_PATH, MV_PATH, NAME, OS, RM_PATH, Unit, UnitKind,
    VIDEO_X,
};

#[derive(Debug, Clone)]
pub struct Delivery {
    pub origin: Arc<str>,
}

impl Delivery {
    pub fn new(origin: String) -> Self {
        Self {
            origin: Arc::from(origin),
        }
    }

    pub fn url_path(self, path: &str) -> String {
        format!("{}{}", self.origin, path)
    }

    pub async fn get_host_os(self) -> Result<String, String> {
        reqwest::Client::new()
            .get(self.url_path(OS))
            .send()
            .await
            .map_err(|x| x.to_string())?
            .json::<String>()
            .await
            .map_err(|x| x.to_string())
    }

    pub async fn get_app_name(self) -> Result<String, String> {
        reqwest::Client::new()
            .get(self.url_path(NAME))
            .send()
            .await
            .map_err(|x| x.to_string())?
            .json::<String>()
            .await
            .map_err(|x| x.to_string())
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

    //FIX : should be available on non wasm target only
    pub async fn iced_upload(&self, _files: &[std::fs::File]) -> Result<(), String> {
        todo!()
    }
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
