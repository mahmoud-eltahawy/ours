use super::Download;
use crate::Message;
use async_recursion::async_recursion;
use common::{Unit, UnitKind};
use delivery::{Delivery, download_file};
use iced::{
    Task,
    widget::{Button, Column, Container, Text},
    window,
};
use std::{
    env::home_dir,
    path::{Path, PathBuf},
};

pub struct Downloads {
    pub state: DownloadingState,
    pub download_dir: PathBuf,
}

pub enum DownloadingState {
    Prepareing {
        units: Vec<Unit>,
    },
    Downloading {
        main_handle: iced::task::Handle,
        tasks_handles: Vec<iced::task::Handle>,
        tasks: Vec<Download>,
    },
    Done,
}

impl Downloads {
    pub fn new() -> Self {
        Self {
            state: DownloadingState::Done,
            download_dir: home_dir().map(|x| x.join("Downloads")).unwrap(),
        }
    }
    pub fn clear(&mut self) {
        self.state = DownloadingState::Done;
    }

    pub fn view(&self) -> Container<'_, Message> {
        match &self.state {
            DownloadingState::Prepareing { units } => Container::new(Column::from_vec(
                units
                    .iter()
                    .map(|x| Text::new(format!("building structures of directory {x:#?}")).into())
                    .collect::<Vec<_>>(),
            )),
            DownloadingState::Downloading {
                main_handle: _,
                tasks_handles,
                tasks,
            } => Container::new({
                let cancel = Button::new("cancel downloads");
                let downloads = Text::new("downloads");
                let buttons = tasks_handles
                    .iter()
                    .zip(tasks)
                    .map(|(_handle, download)| {
                        Button::new(Text::new(format!(
                            "downloading file {:#?}",
                            download.host_path
                        )))
                    })
                    .fold(Column::new(), |acc, x| acc.push(x));
                iced::widget::column![downloads, cancel, buttons]
            }),
            DownloadingState::Done => Container::new(Text::new("done")),
        }
    }
}

#[derive(Debug, Clone)]
pub enum DownloadMessage {
    OpenDownloadWindow,
    DownloadWindowOpened(window::Id),
    CloseDownloadWindow(window::Id),
    DownloadWindowClosed,
    StartDownloading(Vec<Download>),
    DownloadDone,
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
                state.downloads.state = DownloadingState::Prepareing {
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
                if let DownloadingState::Downloading {
                    main_handle,
                    tasks_handles,
                    ..
                } = &state.downloads.state
                {
                    if !main_handle.is_aborted() {
                        main_handle.abort();
                    }
                    for handle in tasks_handles.iter() {
                        if !handle.is_aborted() {
                            handle.abort();
                        }
                    }
                }
                Task::none()
            }
            DownloadMessage::StartDownloading(downloads) => {
                let origin = state.delivery.origin.clone();
                let (mut tasks, handles): (Vec<_>, Vec<_>) = downloads
                    .clone()
                    .iter()
                    .map(move |x| {
                        Task::future(download_file(
                            origin.clone(),
                            x.server_path.clone(),
                            x.host_path.clone(),
                        ))
                        .abortable()
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
                state.downloads.state = DownloadingState::Downloading {
                    main_handle: handle,
                    tasks_handles: handles,
                    tasks: downloads,
                };
                task.map(|x| match x {
                    Ok(_) => Message::Download(DownloadMessage::DownloadDone),
                    Err(err) => Message::ErrorHappned(format!(
                        "error : {err:#?} happened \nwhile downloading"
                    )),
                })
            }
            DownloadMessage::DownloadDone => {
                state.downloads.clear();
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
    Download {
        host_path: pwd.join(unit_path.file_name().unwrap().to_str().unwrap()),
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
