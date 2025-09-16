use super::Download;
use crate::Message;
use async_recursion::async_recursion;
use common::{Unit, UnitKind};
use delivery::Delivery;
use iced::{
    Task,
    futures::StreamExt,
    task::Handle,
    widget::{Button, Column, Container, Text, column},
    window,
};
use reqwest::get;
use std::{
    collections::HashMap,
    env::home_dir,
    hash::{DefaultHasher, Hash, Hasher},
    io,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::io::AsyncWriteExt;

pub struct Downloads {
    pub state: DownloadingState,
    pub download_dir: PathBuf,
    pub downloading: HashMap<u64, Downloading>,
    pub finished: Vec<PathBuf>,
}

pub struct Downloading {
    handle: Handle,
    host_path: PathBuf,
    progress: f32,
}

pub enum DownloadingState {
    MakeDirectories { units: Vec<Unit> },
    Downloading { main_handle: iced::task::Handle },
    Done,
}

impl Downloads {
    pub fn new() -> Self {
        Self {
            state: DownloadingState::Done,
            download_dir: home_dir().map(|x| x.join("Downloads")).unwrap(),
            downloading: HashMap::new(),
            finished: Vec::new(),
        }
    }
    pub fn clear(&mut self) {
        self.state = DownloadingState::Done;
    }

    pub fn view(&self) -> Container<'_, Message> {
        let cancel = Button::new("cancel all downloads");

        let title = Text::new("downloading files");
        let buttons = self
            .downloading
            .values()
            .map(|download| {
                Button::new(Text::new(format!(
                    "downloading file {:#?} with progress {}",
                    download.host_path, download.progress
                )))
            })
            .fold(Column::new(), |acc, x| acc.push(x))
            .spacing(5.);
        let downloading = column![title, buttons];
        let title = Text::new("finished downloads");
        let lines = self
            .finished
            .iter()
            .map(|x| Text::new(format!("{x:#?}")))
            .fold(Column::new(), |acc, x| acc.push(x));
        let finished = column![title, lines];
        Container::new(column![cancel, downloading, finished])
    }
}

#[derive(Debug, Clone)]
pub enum DownloadMessage {
    OpenDownloadWindow,
    DownloadWindowOpened(window::Id),
    CloseDownloadWindow(window::Id),
    DownloadWindowClosed,
    StartDownloading(Vec<Download>),
    Progressing(Progress),
    DownloadDone(u64),
}
use iced::task::{Straw, sipper};

#[derive(Debug, Clone)]
enum Update {
    Downloading(Progress),
    Finished(Result<u64, Error>),
}

pub fn download_file(
    origin: Arc<str>,
    server_path: PathBuf,
    host_path: PathBuf,
) -> impl Straw<(), Progress, Error> {
    sipper(async move |mut progress| {
        println!("file {:#?} started downloading", host_path);
        let end = format!("file {:#?} finished downloading", host_path);
        let url = format!(
            "{}/download/{}",
            origin,
            server_path.to_str().unwrap_or_default()
        );
        let response = get(url).await?;
        let total = response.content_length().ok_or(Error::NoContentLength)?;
        let path_hashed = hash_path(&host_path);
        let _ = progress
            .send(Progress {
                percent: 0.0,
                id: path_hashed,
            })
            .await;

        let mut byte_stream = response.bytes_stream();

        let mut file = tokio::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(&host_path)
            .await?;

        let mut downloaded = 0;
        while let Some(bytes) = byte_stream.next().await {
            let bytes = bytes?;
            downloaded += bytes.len();

            file.write_all(&bytes).await?;

            let _ = progress
                .send(Progress {
                    percent: 100.0 * downloaded as f32 / total as f32,
                    id: path_hashed,
                })
                .await;
        }
        println!("{}", end);

        Ok(())
    })
}

fn hash_path(p: &PathBuf) -> u64 {
    let mut hasher = DefaultHasher::new();
    p.hash(&mut hasher);
    hasher.finish()
}

#[derive(Debug, Clone)]
pub struct Progress {
    pub id: u64,
    pub percent: f32,
}

#[derive(Debug, Clone)]
pub enum Error {
    RequestFailed(Arc<reqwest::Error>),
    WritingBytesFailed(Arc<io::Error>),
    NoContentLength,
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        Error::RequestFailed(Arc::new(error))
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::WritingBytesFailed(Arc::new(error))
    }
}

impl DownloadMessage {
    pub fn handle(self, state: &mut super::ClientState) -> iced::Task<Message> {
        match self {
            DownloadMessage::OpenDownloadWindow => {
                let (_, task) = window::open(window::Settings::default());
                task.map(|id| Message::Download(DownloadMessage::DownloadWindowOpened(id)))
            }
            DownloadMessage::DownloadWindowOpened(id) => {
                state.download_window = Some(id);
                let units = state.select.units.clone();
                state.select.clear();
                state.downloads.state = DownloadingState::MakeDirectories {
                    units: units.clone(),
                };
                Task::perform(
                    prepare_downloads(
                        state.delivery.clone(),
                        units,
                        state.downloads.download_dir.clone(),
                    ),
                    move |x| match x {
                        Ok(downloads) => {
                            Message::Download(DownloadMessage::StartDownloading(downloads))
                        }
                        Err(err) => Message::ErrorHappned(format!(
                            "error : {err:#?} happened\n while preparing for downloading"
                        )),
                    },
                )
            }
            DownloadMessage::CloseDownloadWindow(id) => {
                let task = window::close(id);
                task.map(|_: window::Id| Message::Download(DownloadMessage::DownloadWindowClosed))
            }
            DownloadMessage::DownloadWindowClosed => {
                state.download_window = None;
                if let DownloadingState::Downloading { main_handle, .. } = &state.downloads.state
                    && !main_handle.is_aborted()
                {
                    main_handle.abort();
                }
                Task::none()
            }
            DownloadMessage::StartDownloading(downloads) => {
                let origin = state.delivery.origin.clone();
                let (mut tasks, downloading): (Vec<_>, Vec<_>) = downloads
                    .into_iter()
                    .map(move |x| {
                        let host_path = x.host_path.clone();
                        let host_path2 = x.host_path;
                        let (task, handle) = Task::sip(
                            download_file(origin.clone(), x.server_path.clone(), host_path.clone()),
                            Update::Downloading,
                            move |y| Update::Finished(y.map(|_| hash_path(&host_path))),
                        )
                        .abortable();
                        let d = Downloading {
                            handle,
                            progress: 0.,
                            host_path: host_path2.clone(),
                        };
                        (task, d)
                    })
                    .unzip();
                let (task, handle) = if let Some(last) = tasks.pop() {
                    tasks.reverse();
                    tasks
                        .into_iter()
                        .fold(last, |acc, x| acc.chain(x))
                        .abortable()
                } else {
                    Task::none().abortable()
                };
                state.downloads.downloading = downloading
                    .into_iter()
                    .map(|x| (hash_path(&x.host_path), x))
                    .collect();
                state.downloads.state = DownloadingState::Downloading {
                    main_handle: handle,
                };
                task.map(|x| match x {
                    Update::Downloading(progress) => {
                        Message::Download(DownloadMessage::Progressing(progress))
                    }
                    Update::Finished(Err(err)) => Message::ErrorHappned(format!("{:#?}", err)),
                    Update::Finished(Ok(id)) => Message::Download(Self::DownloadDone(id)),
                })
            }
            DownloadMessage::DownloadDone(id) => {
                if let Some(x) = state.downloads.downloading.remove(&id) {
                    state.downloads.finished.push(x.host_path.clone());
                }
                Task::none()
            }
            DownloadMessage::Progressing(progress) => {
                if let Some(x) = state.downloads.downloading.get_mut(&progress.id) {
                    x.progress = progress.percent;
                };
                Task::none()
            }
        }
    }
}

#[async_recursion]
async fn prepare_downloads(
    delivery: Delivery,
    units: Vec<Unit>,
    download_dir: PathBuf,
) -> Result<Vec<Download>, String> {
    let mut res = Vec::new();
    for unit in units.into_iter() {
        match unit.kind {
            UnitKind::Dirctory => {
                res.extend(prepare_directory(delivery.clone(), unit, &download_dir).await?);
            }
            _ => {
                res.push(prepare_file(unit.path, &download_dir));
            }
        }
    }
    Ok(res)
}

fn prepare_file(unit_path: PathBuf, pwd: &Path) -> Download {
    let host_path = pwd.join(unit_path.file_name().unwrap().to_str().unwrap());
    Download {
        id: hash_path(&host_path),
        host_path,
        server_path: unit_path,
    }
}

pub async fn prepare_directory(
    delivery: Delivery,
    unit: Unit,
    pwd: &Path,
) -> Result<Vec<Download>, String> {
    let inner_units = delivery.clone().ls(unit.path.clone()).await?;
    let pwd = pwd.join(unit.name());
    tokio::fs::create_dir(&pwd)
        .await
        .map_err(|x| x.to_string())?;
    prepare_downloads(delivery.clone(), inner_units, pwd).await
}

// struct VirtualFolder {
//     path: PathBuf,
//     files: Vec<UnitPaths>,
//     folders: Vec<VirtualFolder>,
// }

// struct UnitPaths {
//     server_path: PathBuf,
//     host_path: PathBuf,
// }

// impl VirtualFolder {
//     fn new_empty(path: PathBuf) -> Self {
//         Self {
//             path,
//             files: Vec::new(),
//             folders: Vec::new(),
//         }
//     }

//     fn push(&mut self, unit: Unit) {
//         let host_path = self.path.join(unit.name());
//         let Unit { path, kind } = unit;
//         match kind {
//             UnitKind::Dirctory => self.folders.push(VirtualFolder::new_empty(host_path)),
//             _ => self.files.push(UnitPaths {
//                 server_path: path,
//                 host_path,
//             }),
//         }
//     }
//     fn extend(&mut self, units: Vec<Unit>) {
//         for unit in units.into_iter() {
//             self.push(unit);
//         }
//     }

//     fn downloads_folder(units: Vec<Unit>) -> Self {
//         let path = home_dir().map(|x| x.join("Downloads")).unwrap();
//         let mut res = Self::new_empty(path);
//         res.extend(units);
//         res
//     }
//     // fn files_downloads_futures(
//     //     &self,
//     //     delivery: Delivery,
//     // ) -> Vec<impl Future<Output = Result<(), String>>> {
//     //     self.files
//     //         .iter()
//     //         .map(|file| {
//     //             download_file(
//     //                 delivery.origin.clone(),
//     //                 file.server_path.clone(),
//     //                 file.host_path.clone(),
//     //             )
//     //         })
//     //         .collect()
//     // }
// }
