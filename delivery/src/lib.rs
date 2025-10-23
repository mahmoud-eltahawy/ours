use std::{path::PathBuf, sync::Arc};

use grpc::client::Unit;

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

    pub async fn ls(self, _base: PathBuf) -> Result<Vec<Unit>, String> {
        //TODO : repalce it with grpc
        todo!()
    }
}
