use super::nav::nav_service_server::NavService;
use crate::nav::{
    DownloadRequest, DownloadResponse, FileSizeRequest, FileSizeResponse, ResumeDownloadRequest,
    ResumeDownloadResponse, UploadRequest, UploadResponse,
};
use crate::{
    error::RpcError,
    nav::{LsRequest, LsResponse, Unit, UnitKind, nav_service_server::NavServiceServer},
};
use common::{AUDIO_X, VIDEO_X};
use std::io::SeekFrom;
use std::pin::Pin;
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::PathBuf,
};
use tokio::fs::{self, File};
use tokio::io::{AsyncReadExt, AsyncSeekExt, BufReader};
use tokio::sync::mpsc;
use tokio_stream::Stream;
use tokio_stream::wrappers::ReceiverStream;
use tonic::Streaming;
use tonic::{Request, Response, Status, async_trait, transport::Server};

pub struct RpcServer {
    pub target_dir: PathBuf,
    pub port: u16,
}

#[async_trait]
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
            let path = x.path();
            let Ok(path) = path.strip_prefix(&self.target_dir) else {
                continue;
            };
            let unit = Unit {
                path: path.to_str().unwrap().to_string(),
                kind: kind.into(),
            };
            units.push(unit);
        }
        Ok(Response::new(LsResponse { units }))
    }

    async fn file_size(
        &self,
        req: Request<FileSizeRequest>,
    ) -> Result<Response<FileSizeResponse>, Status> {
        let Ok(path) = req.into_inner().path.parse::<PathBuf>();
        let path = self.target_dir.join(path);
        let len = File::open(path).await?.metadata().await?.len();
        Ok(Response::new(FileSizeResponse { size: len }))
    }

    type DownloadStream = Pin<Box<dyn Stream<Item = Result<DownloadResponse, Status>> + Send>>;
    async fn download(
        &self,
        req: Request<DownloadRequest>,
    ) -> Result<Response<Self::DownloadStream>, Status> {
        let Ok(path) = req.into_inner().path.parse::<PathBuf>();
        let path = self.target_dir.join(path);
        let file = File::open(path).await?;
        let mut file = BufReader::new(file);
        let (tx, rx) = mpsc::channel::<Result<DownloadResponse, Status>>(1024);
        tokio::spawn(async move {
            loop {
                let mut buffer = bytes::BytesMut::with_capacity(1024);
                let rb = match file.read_buf(&mut buffer).await {
                    Ok(rb) => rb,
                    Err(err) => {
                        return tx.send(Err(err.into())).await;
                    }
                };
                if rb == 0 {
                    break;
                }
                tx.send(Ok(DownloadResponse {
                    data: buffer.to_vec(),
                }))
                .await?;
            }
            Ok(())
        });
        let output_stream = ReceiverStream::new(rx);
        Ok(Response::new(
            Box::pin(output_stream) as Self::DownloadStream
        ))
    }

    type ResumeDownloadStream =
        Pin<Box<dyn Stream<Item = Result<ResumeDownloadResponse, Status>> + Send>>;
    async fn resume_download(
        &self,
        req: Request<ResumeDownloadRequest>,
    ) -> Result<Response<Self::ResumeDownloadStream>, Status> {
        let ResumeDownloadRequest {
            path,
            progress_index,
        } = req.into_inner();
        let Ok(path) = path.parse::<PathBuf>();
        let path = self.target_dir.join(path);
        let file = File::open(path).await?;
        let mut file = BufReader::new(file);
        file.seek(SeekFrom::Start(progress_index)).await?;
        let (tx, rx) = mpsc::channel::<Result<ResumeDownloadResponse, Status>>(1024);
        tokio::spawn(async move {
            loop {
                let mut buffer = bytes::BytesMut::with_capacity(1024);
                let rb = match file.read_buf(&mut buffer).await {
                    Ok(rb) => rb,
                    Err(err) => {
                        return tx.send(Err(err.into())).await;
                    }
                };
                if rb == 0 {
                    break;
                }
                tx.send(Ok(ResumeDownloadResponse {
                    data: buffer.to_vec(),
                }))
                .await?;
            }
            Ok(())
        });
        let output_stream = ReceiverStream::new(rx);
        Ok(Response::new(
            Box::pin(output_stream) as Self::ResumeDownloadStream
        ))
    }

    async fn upload(
        &self,
        req: Request<Streaming<UploadRequest>>,
    ) -> Result<Response<UploadResponse>, Status> {
        unimplemented!()
    }
}

impl RpcServer {
    pub fn new(target_dir: PathBuf, port: u16) -> Self {
        Self { target_dir, port }
    }
    pub async fn serve(self) -> Result<(), RpcError> {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), self.port);
        Server::builder()
            .add_service(NavServiceServer::new(self))
            .serve(addr)
            .await?;

        Ok(())
    }
}
