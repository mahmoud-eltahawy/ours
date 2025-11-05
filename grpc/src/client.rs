use crate::{
    error::RpcError,
    nav::{DownloadRequest, DownloadResponse, LsRequest, nav_service_client::NavServiceClient},
    top,
};
use std::{env::home_dir, net::SocketAddr, path::PathBuf, sync::Arc};
use tokio::{
    fs::{File, create_dir_all},
    io::AsyncWriteExt,
    sync::Mutex,
};
use tonic::transport::Channel;

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

    pub async fn download_file(self, target: PathBuf) -> Result<(), RpcError> {
        let req = DownloadRequest {
            path: target.to_str().unwrap().to_string(),
        };
        let mut client = self.client.lock().await;
        let mut stream = client.download(req).await?.into_inner();
        let target = home_dir().unwrap().join("Downloads").join(&target);
        create_dir_all(target.parent().map(|x| x.to_path_buf()).unwrap_or_default()).await?;
        let mut file = File::create(target).await?;
        while let Some(DownloadResponse { data }) = stream.message().await? {
            file.write_all(&data).await?;
            file.flush().await?;
        }
        Ok(())
    }
}
