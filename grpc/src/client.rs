use crate::{
    error::RpcError,
    nav::{
        DownloadRequest, FileSizeRequest, LsRequest, ResumeDownloadRequest, UploadMetadata,
        UploadRequest, nav_service_client::NavServiceClient, upload_request::Data,
    },
    top,
};
use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::{
    fs::File,
    io::{AsyncReadExt, BufReader},
    sync::{Mutex, mpsc},
};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Streaming, transport::Channel};

pub use crate::nav::{DownloadResponse, ResumeDownloadResponse};

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
    ) -> Result<Streaming<ResumeDownloadResponse>, RpcError> {
        let path = target.to_str().unwrap().to_string();
        let req = ResumeDownloadRequest {
            path: path.clone(),
            progress_index: progress_index as u64,
        };
        let mut client = self.client.lock().await;
        let stream = client.resume_download(req).await?.into_inner();
        Ok(stream)
    }

    pub async fn upload(
        self,
        location_path: PathBuf,
        target_path: PathBuf,
    ) -> Result<(), RpcError> {
        let tpath = target_path.to_str().unwrap().to_string();
        let location_path = location_path.to_str().unwrap().to_string();
        let init_req = UploadRequest {
            data: Some(Data::Meta(UploadMetadata {
                target_path: tpath,
                location_path,
            })),
        };
        let (tx, rx) = mpsc::channel::<UploadRequest>(100);
        let mut client = self.client.lock().await;
        let _ = client.upload(ReceiverStream::new(rx)).await?.into_inner();
        let _ = tx.send(init_req).await;

        let file = File::open(&target_path).await?;
        let mut file = BufReader::new(file);

        let mut buffer = bytes::BytesMut::with_capacity(1024 * 1024);
        let res = loop {
            let rb = match file.read_buf(&mut buffer).await {
                Ok(rb) => rb,
                Err(err) => {
                    eprintln!(
                        "ERROR : upload of {path} due to -> {err}",
                        path = target_path.display()
                    );
                    match tx.send(UploadRequest { data: None }).await {
                        Ok(_) => (),
                        Err(err) => break Err(err),
                    };
                    break Ok(());
                }
            };
            if rb == 0 {
                break Ok(());
            }
            match tx
                .send(UploadRequest {
                    data: Some(Data::Chunk(buffer.to_vec())),
                })
                .await
            {
                Ok(_) => (),
                Err(err) => break Err(err),
            };
            buffer.clear();
        };

        match res {
            Ok(_) => Ok(()),
            Err(err) => Err(RpcError::from(err.to_string())),
        }
    }
}
