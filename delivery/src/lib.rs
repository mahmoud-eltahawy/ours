use std::path::PathBuf;

use common::MP4_PATH;

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
}
