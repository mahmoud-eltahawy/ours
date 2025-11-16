use crate::{
    error::RpcError,
    nav::{DownloadRequest, FileSizeRequest, LsRequest, nav_service_client::NavServiceClient},
    top,
};
use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::sync::Mutex;
use tonic::{Streaming, transport::Channel};

pub use crate::nav::DownloadResponse;

#[derive(Clone, Debug)]
pub struct RpcClient {
    pub addr: SocketAddr,
    pub client: Arc<Mutex<NavServiceClient<Channel>>>,
}

impl RpcClient {
    pub async fn new(addr: SocketAddr) -> Result<Self, RpcError> {
        let client: NavServiceClient<Channel> =
            NavServiceClient::connect(format!("http://{}", addr)).await?;
        let client = Arc::new(Mutex::new(client));
        Ok(Self { addr, client })
    }

    pub async fn ls(self, target: PathBuf) -> Result<Vec<top::Unit>, RpcError> {
        let req = LsRequest {
            path: target.to_str().unwrap().to_string(),
        };
        let mut client = self.client.lock().await;
        let mut units: Vec<top::Unit> = client
            .ls(req)
            .await?
            .into_inner()
            .units
            .into_iter()
            .map(|x| {
                let Ok(path) = x.path.parse::<PathBuf>();
                top::Unit {
                    path,
                    kind: x.kind(),
                }
            })
            .collect();
        units.sort_by_key(|x| (x.kind, x.name()));
        Ok(units)
    }

    pub async fn download_stream(
        self,
        target: &Path,
    ) -> Result<(u64, Streaming<DownloadResponse>), RpcError> {
        let path = target.to_str().unwrap().to_string();
        let req = DownloadRequest { path: path.clone() };
        let mut client = self.client.lock().await;
        let stream = client.download(req).await?.into_inner();
        let req = FileSizeRequest { path };
        let size = client.file_size(req).await?.into_inner().size;
        Ok((size, stream))
    }

    pub async fn resume_stream(
        self,
        progress_index: usize,
        target: &Path,
    ) -> Result<Streaming<DownloadResponse>, RpcError> {
        todo!()
    }
}
