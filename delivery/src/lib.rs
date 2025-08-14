use std::path::PathBuf;

use common::{CP_PATH, LS_PATH, MKDIR_PATH, MP4_PATH, MV_PATH, RM_PATH, Unit};
use gloo::net::http::Request;

#[derive(Debug, Clone)]
pub struct Delivery {
    origin: String,
}

impl Delivery {
    pub fn new(origin: String) -> Self {
        Self { origin }
    }

    fn url_path(&self, path: &str) -> String {
        format!("{}{}", self.origin, path)
    }

    pub async fn mp4_remux(&self, targets: Vec<PathBuf>) -> Result<(), String> {
        reqwest::Client::new()
            .post(self.url_path(MP4_PATH))
            .json(&targets)
            .send()
            .await
            .map_err(|x| x.to_string())?;
        Ok(())
    }

    pub async fn cp(&self, targets: Vec<PathBuf>, to: PathBuf) -> Result<(), String> {
        reqwest::Client::new()
            .post(self.url_path(CP_PATH))
            .json(&(targets, to))
            .send()
            .await
            .map_err(|x| x.to_string())?;
        Ok(())
    }

    pub async fn mv(&self, targets: Vec<PathBuf>, to: PathBuf) -> Result<(), String> {
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
            .map_err(|x| x.to_string())?;
        Ok(res)
    }

    pub async fn mkdir(&self, target: PathBuf) -> Result<(), String> {
        let _ = reqwest::Client::new()
            .post(self.url_path(MKDIR_PATH))
            .json(&target)
            .send()
            .await
            .map_err(|x| x.to_string())?;
        Ok(())
    }

    pub async fn rm(&self, bases: Vec<Unit>) -> Result<(), String> {
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

    //FIX : should be available on nonw wasm target only
    pub async fn iced_upload(&self, _files: &[std::fs::File]) -> Result<(), String> {
        todo!()
    }
}
