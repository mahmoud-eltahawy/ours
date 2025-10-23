use super::nav::nav_service_server::NavService;
use crate::{
    RpcError,
    nav::{LsRequest, LsResponse, Unit, UnitKind, nav_service_server::NavServiceServer},
};
use common::{AUDIO_X, VIDEO_X};
use std::path::PathBuf;
use tokio::fs;
use tonic::{Request, Response, Status, transport::Server};

pub struct RpcServer {
    pub target_dir: PathBuf,
    pub port: u16,
}

#[tonic::async_trait]
impl NavService for RpcServer {
    async fn ls(&self, req: Request<LsRequest>) -> Result<Response<LsResponse>, Status> {
        let Ok(root) = req.into_inner().path.parse::<PathBuf>();
        let root = self.target_dir.join(root);
        let mut dir = fs::read_dir(&root).await?;
        let mut units = Vec::new();
        while let Some(x) = dir.next_entry().await? {
            let kind = if x.file_type().await?.is_dir() {
                UnitKind::Folder
            } else {
                let ex = x.path();
                let ex = ex.extension().and_then(|x| x.to_str());
                match ex {
                    Some(ex) => {
                        if VIDEO_X.contains(&ex) {
                            UnitKind::Video
                        } else if AUDIO_X.contains(&ex) {
                            UnitKind::Audio
                        } else {
                            UnitKind::File
                        }
                    }
                    _ => UnitKind::File,
                }
            };
            let unit = Unit {
                path: x.path().to_str().unwrap().to_string(),
                kind: kind.into(),
            };
            units.push(unit);
        }
        Ok(Response::new(LsResponse { units }))
    }
}

impl RpcServer {
    pub fn new(target_dir: PathBuf, port: u16) -> Self {
        Self { target_dir, port }
    }
    pub async fn serve(&self) -> Result<(), RpcError> {
        let Self { target_dir, port } = self;
        let rpc_service = RpcServer {
            target_dir: target_dir.to_path_buf(),
            port: *port,
        };

        let addr = format!("[::1]:{port}").parse()?;
        Server::builder()
            .add_service(NavServiceServer::new(rpc_service))
            .serve(addr)
            .await?;

        Ok(())
    }
}
