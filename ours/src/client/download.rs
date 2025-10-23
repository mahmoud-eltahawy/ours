use super::Download;
use crate::{Message, client::VIOLET};
use async_recursion::async_recursion;
use delivery::Delivery;
use grpc::{UnitKind, client::Unit};
use iced::{
    Alignment, Border, Color, Element, Length, Task,
    border::Radius,
    futures::StreamExt,
    task::Handle,
    widget::{Button, Column, Container, Text, column, progress_bar, row, scrollable},
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
    time::Duration,
};
use tokio::{io::AsyncWriteExt, time::sleep};

pub struct Downloads {
    state: DownloadingState,
    download_dir: PathBuf,
    pub downloading: HashMap<u64, Downloading>,
    finished: Vec<PathBuf>,
    failed: HashMap<u64, FailedDownload>,
}

struct FailedDownload {
    download: Downloading,
    error: Error,
}

#[derive(Clone)]
pub struct Downloading {
    handle: Handle,
    size: u64,
    host_path: PathBuf,
    progress_state: ProgressState,
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
            failed: HashMap::new(),
        }
    }

    pub fn clear(&mut self) {
        *self = Self::new();
    }

    pub fn view(&self) -> Element<'_, Message> {
        let cancel = Button::new("cancel all downloads")
            .on_press(Message::Download(DownloadMessage::AbortDownloads));

        let downloading = self.downloading();
        let finished = self.finished();
        let failed = self.failed();
        Container::new(column![cancel, downloading, finished, failed].spacing(40.)).into()
    }

    fn failed(&self) -> scrollable::Scrollable<'_, Message> {
        let lines = self
            .failed
            .values()
            .map(|x| {
                format!(
                    "download {:#?},failed because of {:#?}",
                    x.download.host_path, x.error
                )
            })
            .map(Text::new)
            .fold(Column::new(), |acc, x| acc.push(x));
        let title =
            (!self.failed.is_empty()).then_some(Text::new("failed downloads").size(50.).center());
        scrollable(column![title, lines])
    }

    fn finished(&self) -> scrollable::Scrollable<'_, Message> {
        let lines = self
            .finished
            .iter()
            .map(|x| format!("{x:#?}"))
            .map(Text::new)
            .fold(Column::new(), |acc, x| acc.push(x));
        let title = (!self.finished.is_empty())
            .then_some(Text::new("finished downloads").size(50.).center());
        scrollable(column![title, lines])
    }

    fn downloading(&self) -> scrollable::Scrollable<'_, Message> {
        let buttons = self
            .downloading
            .values()
            .map(download_bar)
            .fold(Column::new(), |acc, x| acc.push(x))
            .spacing(5.);
        let title = (!self.downloading.is_empty())
            .then_some(Text::new("downloading files").size(50.).center());
        scrollable(column![title, buttons])
    }
}

fn download_bar(x: &Downloading) -> Column<'_, Message> {
    let downloaded = match x.progress_state {
        ProgressState::Waiting | ProgressState::Started { .. } => 0.0,
        ProgressState::Marshing { downloaded } => downloaded as f32,
        ProgressState::Finished => x.size as f32,
    };
    let bar = progress_bar(0.0..=(x.size as f32), downloaded)
        .length(Length::Fill)
        .style(move |_| {
            if downloaded == 0.0 {
                let color = Color::from_rgb(VIOLET.g, VIOLET.b, VIOLET.r);
                progress_bar::Style {
                    background: iced::Background::Color(color),
                    bar: iced::Background::Color(color),
                    border: Border {
                        color,
                        width: 4.,
                        radius: Radius::new(100.),
                    },
                }
            } else {
                progress_bar::Style {
                    background: iced::Background::Color(Color::BLACK),
                    bar: iced::Background::Color(VIOLET),
                    border: Border {
                        color: VIOLET,
                        width: 7.,
                        radius: Radius::new(7.),
                    },
                }
            }
        });
    let text = if x.size == 0 {
        format!("file {:#?} is pending", x.host_path)
    } else {
        format!(
            "downloading file {:#?} of {} space",
            x.host_path,
            proper_size(x.size)
        )
    };
    let title = Text::new(text).size(20.).center();
    let cancel = Text::new("cancel").size(20.).center();
    let cancel = Button::new(cancel).on_press(Message::Download(DownloadMessage::AbortDownload(
        hash_path(&x.host_path),
        x.handle.clone(),
    )));
    let head = row![title, cancel].spacing(10.);
    column![head, bar]
        .align_x(Alignment::Center)
        .spacing(3.)
        .padding(20.)
}

fn proper_size(x: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * KB;
    const GB: u64 = MB * MB;

    if (0..KB).contains(&x) {
        format!("{x} B")
    } else if (KB..MB).contains(&x) {
        format!("{} KB", x / KB)
    } else if (MB..GB).contains(&x) {
        format!("{} MB", x / MB)
    } else {
        format!("{} GB", x / GB)
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
    DownloadFailed(Error, u64),
    AbortDownloads,
    AbortDownload(u64, Handle),
}
use iced::task::{Straw, sipper};

#[derive(Debug, Clone)]
enum Update {
    Downloading(Progress),
    Finished(u64),
    Failed(Error, u64),
}

pub fn download_file(
    origin: Arc<str>,
    server_path: PathBuf,
    host_path: PathBuf,
) -> impl Straw<(), Progress, Error> {
    sipper(async move |mut progress| {
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
                id: path_hashed,
                progress_state: ProgressState::Started { total },
            })
            .await;

        let mut byte_stream = response.bytes_stream();

        let _ = tokio::fs::remove_file(&host_path).await;
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

            #[cfg(debug_assertions)]
            sleep(Duration::from_millis(30)).await;

            let _ = progress
                .send(Progress {
                    id: path_hashed,
                    progress_state: ProgressState::Marshing { downloaded },
                })
                .await;
        }
        let _ = progress
            .send(Progress {
                id: path_hashed,
                progress_state: ProgressState::Finished,
            })
            .await;

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
    pub progress_state: ProgressState,
}
#[derive(Debug, Clone, Default)]
pub enum ProgressState {
    #[default]
    Waiting,
    Started {
        total: u64,
    },
    Marshing {
        downloaded: usize,
    },
    Finished,
}

#[derive(Debug, Clone)]
pub enum Error {
    RequestFailed(Arc<reqwest::Error>),
    WritingBytesFailed(Arc<io::Error>),
    NoContentLength,
    AbortedByUser,
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
                let (tasks, downloading): (Vec<_>, Vec<_>) = downloads
                    .into_iter()
                    .map(move |x| {
                        let host_path = x.host_path.clone();
                        let (task, handle) = Task::sip(
                            download_file(origin.clone(), x.server_path.clone(), host_path.clone()),
                            Update::Downloading,
                            move |y| match y {
                                Ok(_) => Update::Finished(x.id),
                                Err(err) => Update::Failed(err, x.id),
                            },
                        )
                        .abortable();
                        (
                            task,
                            Downloading {
                                handle,
                                size: 0,
                                progress_state: ProgressState::Waiting,
                                host_path,
                            },
                        )
                    })
                    .unzip();

                let (task, handle) = group_tasks(tasks).abortable();
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
                    Update::Finished(id) => Message::Download(Self::DownloadDone(id)),
                    Update::Failed(err, id) => Message::Download(Self::DownloadFailed(err, id)),
                })
            }
            DownloadMessage::DownloadDone(id) => {
                if let Some(x) = state.downloads.downloading.remove(&id) {
                    state.downloads.finished.push(x.host_path.clone());
                }
                Task::none()
            }
            DownloadMessage::DownloadFailed(error, id) => {
                if let Some(download) = state.downloads.downloading.remove(&id) {
                    state
                        .downloads
                        .failed
                        .insert(id, FailedDownload { download, error });
                }
                Task::none()
            }
            DownloadMessage::Progressing(progress) => {
                if let Some(x) = state.downloads.downloading.get_mut(&progress.id) {
                    if let ProgressState::Started { total } = progress.progress_state {
                        x.size = total;
                    }
                    x.progress_state = progress.progress_state;
                };
                Task::none()
            }
            DownloadMessage::AbortDownloads => {
                if let DownloadingState::Downloading { main_handle } = &state.downloads.state {
                    main_handle.abort();
                }
                state
                    .downloads
                    .failed
                    .extend(state.downloads.downloading.values().map(|x| {
                        (
                            hash_path(&x.host_path),
                            FailedDownload {
                                download: x.clone(),
                                error: Error::AbortedByUser,
                            },
                        )
                    }));
                state.downloads.downloading.clear();
                Task::none()
            }
            DownloadMessage::AbortDownload(id, handle) => {
                if let Some(x) = state.downloads.downloading.remove(&id) {
                    state.downloads.failed.insert(
                        id,
                        FailedDownload {
                            download: x,
                            error: Error::AbortedByUser,
                        },
                    );
                };
                handle.abort();
                Task::none()
            }
        }
    }
}

fn group_tasks(tasks: Vec<Task<Update>>) -> Task<Update> {
    const CAP: usize = 4;
    let mut vec_vec = Vec::new();
    let mut vec = Vec::with_capacity(CAP);
    for t in tasks.into_iter() {
        if vec.len() < CAP {
            vec.push(t);
        } else {
            vec_vec.push(vec);
            vec = Vec::with_capacity(CAP);
        }
    }
    vec_vec
        .into_iter()
        .map(Task::batch)
        .fold(Task::none(), |acc, x| acc.chain(x))
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
            UnitKind::Folder => {
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
    let _ = tokio::fs::create_dir_all(&pwd).await;
    prepare_downloads(delivery.clone(), inner_units, pwd).await
}
