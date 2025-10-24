use crate::{
    error::RpcError,
    nav::{LsRequest, nav_service_client::NavServiceClient},
    top,
};
use std::{net::SocketAddr, path::PathBuf, sync::Arc};
use tokio::sync::Mutex;
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
        let units = client
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
        Ok(units)
    }
}
