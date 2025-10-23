use std::{path::PathBuf, sync::Arc};

use common::{AUDIO_X, LS_PATH, Unit, UnitKind, VIDEO_X};

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
