use crate::{Message, State, client::svg_button};
use common::assets::IconName;
use grpc::{
    UnitKind,
    client::{DownloadResponse, RpcClient},
    error::RpcError,
    top::Unit,
};
use iced::{
    Alignment, Background, Border, Element, Length, Task, Theme,
    border::Radius,
    task::{Handle, Straw, sipper},
    widget::{Column, Container, Text, column, container, progress_bar, row, scrollable},
};
use std::{
    env::home_dir,
    path::{Path, PathBuf},
};
use tokio::{
    fs::{File, create_dir_all, remove_file},
    io::AsyncWriteExt,
};

#[derive(Default, Debug)]
pub struct Downloads {
    pub show_preview: bool,
    progressing_count: usize,
    waiting_count: usize,
    finished_count: usize,
    failed_count: usize,
    canceled_count: usize,
    files: Vec<Download>,
}

#[derive(Clone)]
pub enum DownloadMessage {
    TogglePreview,
    QueueFromSelectedStart,
    QueueFromSelected(Result<Vec<PathBuf>, RpcError>),
    Tick(DownloadProgress),
    CancelProgress(usize, Handle),
    ProgressCanceled(usize),
    UpgradePriorty(usize),
    DowngradePriorty(usize),
    RetryCanceled(usize),
    RetryFailed(usize),
}

impl State {
    pub fn handle_downloads_msg(&mut self, msg: DownloadMessage) -> Task<Message> {
        let state = &mut self.client;
        match msg {
            DownloadMessage::QueueFromSelectedStart => {
                let units = state.select.units.clone();
                let Some(grpc) = &state.grpc else {
                    return Task::none();
                };
                state.select.clear();
                let fut = get_download_paths(grpc.clone(), units);
                Task::perform(fut, |x| DownloadMessage::QueueFromSelected(x).into())
            }
            DownloadMessage::QueueFromSelected(paths) => {
                let paths = match paths {
                    Ok(paths) => paths,
                    Err(err) => {
                        dbg!(err);
                        return Task::none();
                    }
                };
                state.downloads.wait(paths);

                let Some(grpc) = state.grpc.clone() else {
                    return Task::none();
                };

                state.downloads.tick_available(grpc)
            }
            DownloadMessage::Tick(download_progress) => match download_progress {
                DownloadProgress::Begin { index, total_size } => {
                    state.downloads.files[index].total_size = total_size as usize;
                    Task::none()
                }
                DownloadProgress::Progressed { index, by } => {
                    state.downloads.files[index].sended += by;
                    Task::none()
                }
                DownloadProgress::Finish(index) => {
                    state.downloads.finish(index);
                    Task::none()
                }
                DownloadProgress::Close { index, result } => {
                    if let Err(err) = result {
                        state.downloads.fail(index, err);
                    }
                    let Some(grpc) = state.grpc.clone() else {
                        return Task::none();
                    };
                    state.downloads.tick_available(grpc)
                }
            },
            DownloadMessage::TogglePreview => {
                state.downloads.show_preview = !state.downloads.show_preview;
                Task::none()
            }
            DownloadMessage::CancelProgress(index, handle) => {
                handle.abort();
                Task::perform(
                    remove_file(join_downloads(&state.downloads.files[index].path)),
                    move |_| DownloadMessage::ProgressCanceled(index).into(),
                )
            }
            DownloadMessage::ProgressCanceled(index) => {
                state.downloads.cancel(index);
                let Some(grpc) = state.grpc.clone() else {
                    return Task::none();
                };
                state.downloads.tick_available(grpc)
            }
            DownloadMessage::UpgradePriorty(index) => {
                state.downloads.upgrade_waiting(index);
                Task::none()
            }
            DownloadMessage::DowngradePriorty(index) => {
                state.downloads.downgrade_waiting(index);
                Task::none()
            }
            DownloadMessage::RetryCanceled(index) => {
                state.downloads.retry_canceled(index);
                let Some(grpc) = state.grpc.clone() else {
                    return Task::none();
                };
                state.downloads.tick_available(grpc)
            }
            DownloadMessage::RetryFailed(index) => {
                state.downloads.retry_failed(index);
                let Some(grpc) = state.grpc.clone() else {
                    return Task::none();
                };
                state.downloads.tick_available(grpc)
            }
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct Download {
    path: PathBuf,
    state: DownloadState,
    total_size: usize,
    sended: usize,
}

impl From<PathBuf> for Download {
    fn from(value: PathBuf) -> Self {
        Self {
            path: value,
            state: DownloadState::Waiting,
            total_size: 0,
            sended: 0,
        }
    }
}

#[derive(Default, Debug, Clone)]
pub enum DownloadState {
    #[default]
    Waiting,
    Progressing(Handle),
    Finished,
    Failed(RpcError),
    Canceled,
}

impl Downloads {
    fn wait(&mut self, paths: Vec<PathBuf>) {
        self.waiting_count += paths.len();
        self.files.extend(paths.into_iter().map(Download::from));
    }
    pub fn active_count(&self) -> usize {
        self.progressing_count + self.waiting_count
    }
    fn is_upgradable(&self, index: usize) -> bool {
        self.files
            .get(index - 1)
            .is_some_and(|x| matches!(x.state, DownloadState::Waiting))
    }
    fn is_downgradable(&self, index: usize) -> bool {
        self.files
            .get(index + 1)
            .is_some_and(|x| matches!(x.state, DownloadState::Waiting))
    }
    fn upgrade_waiting(&mut self, index: usize) {
        if !self.is_upgradable(index) {
            return;
        }
        let temp = self.files[index].clone();
        self.files[index] = self.files[index - 1].clone();
        self.files[index - 1] = temp;
    }
    fn downgrade_waiting(&mut self, index: usize) {
        if !self.is_downgradable(index) {
            return;
        }
        let temp = self.files[index].clone();
        self.files[index] = self.files[index + 1].clone();
        self.files[index + 1] = temp;
    }
    fn progress(&mut self, index: usize, handle: Handle) {
        self.waiting_count -= 1;
        self.progressing_count += 1;
        self.files[index].state = DownloadState::Progressing(handle);
    }
    fn finish(&mut self, index: usize) {
        self.progressing_count -= 1;
        self.finished_count += 1;
        self.files[index].state = DownloadState::Finished;
    }
    fn fail(&mut self, index: usize, err: RpcError) {
        self.progressing_count -= 1;
        self.failed_count += 1;
        self.files[index].state = DownloadState::Failed(err);
    }

    fn retry_canceled(&mut self, index: usize) {
        self.waiting_count += 1;
        self.canceled_count -= 1;
        self.files[index].state = DownloadState::Waiting;
    }

    fn retry_failed(&mut self, index: usize) {
        self.waiting_count += 1;
        self.failed_count -= 1;
        self.files[index].state = DownloadState::Waiting;
    }

    fn cancel(&mut self, index: usize) {
        self.progressing_count -= 1;
        self.canceled_count += 1;
        self.files[index].state = DownloadState::Canceled;
        self.files[index].sended = 0;
    }
    fn first_waiting(&mut self) -> Option<(&Download, usize)> {
        for (i, value) in self.files.iter().enumerate() {
            if matches!(value.state, DownloadState::Waiting) {
                return Some((value, i));
            }
        }
        None
    }
    fn fill_turn(&mut self, grpc: RpcClient) -> Option<Task<DownloadProgress>> {
        if self.progressing_count >= 5 {
            return None;
        }
        match self.first_waiting() {
            Some((download, index)) => {
                let (task, handle) = Task::sip(
                    download_file(grpc.clone(), index, download.path.clone()),
                    |progress| progress,
                    move |x| DownloadProgress::Close { index, result: x },
                )
                .abortable();

                self.progress(index, handle);
                Some(task)
            }
            None => None,
        }
    }

    fn tick_available(&mut self, grpc: RpcClient) -> Task<Message> {
        let mut xs = Vec::new();

        while let Some(task) = self.fill_turn(grpc.clone()).map(|task| {
            task.map(move |download_progress| DownloadMessage::Tick(download_progress).into())
        }) {
            xs.push(task);
        }
        Task::batch(xs)
    }

    pub fn view(&self) -> Element<'_, Message> {
        let title = Text::new("Downloads");
        let progressing = self.progressing_view();
        let waiting = self.waiting_view();
        let failed = self.failed_view();
        let finished = self.finished_view();
        let canceled = self.canceled_view();
        let content = scrollable(
            column![title, progressing, waiting, failed, finished, canceled]
                .align_x(Alignment::Center)
                .spacing(20.),
        );
        Container::new(content)
            .style(|theme: &Theme| container::Style {
                border: Border {
                    width: 2.,
                    color: theme.palette().primary,
                    radius: Radius::new(8.),
                },
                background: Some(Background::Color(theme.palette().background)),
                ..Default::default()
            })
            .padding(12.)
            .into()
    }

    fn progressing_view(&self) -> Option<Element<'_, Message>> {
        if self.progressing_count == 0 {
            return None;
        }

        fn format_size(x: usize) -> String {
            const KB: usize = 1024;
            const MB: usize = KB * KB;
            const GB: usize = MB * MB;

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

        let title = Text::new("in progress downloads");
        let content = column![title];
        let content = self
            .files
            .iter()
            .enumerate()
            .filter_map(|(index, download)| match &download.state {
                DownloadState::Progressing(handle) => {
                    let txt = Text::new(format!(
                        "{:#?}, {} of {},{:.2}%",
                        download.path,
                        format_size(download.sended),
                        format_size(download.total_size),
                        (download.sended as f32 / download.total_size as f32) * 100.0
                    ));
                    let progress_bar =
                        progress_bar(0.0..=(download.total_size as f32), download.sended as f32);
                    let left = column![txt, progress_bar].align_x(Alignment::Center);
                    let button = svg_button(IconName::Close.get())
                        .height(Length::Fixed(80.))
                        .clip(false)
                        .on_press(DownloadMessage::CancelProgress(index, handle.clone()).into());
                    let row = row![left, button].align_y(Alignment::Center).spacing(5.);
                    Some(row)
                }
                _ => None,
            })
            .fold(content, |acc, x| acc.push(x));
        let content = scrollable(content.spacing(3.));
        Some(content.into())
    }

    fn waiting_view(&self) -> Option<Element<'_, Message>> {
        if self.waiting_count == 0 {
            return None;
        }
        let title = Text::new("waiting downloads");
        let content = column![title];
        let content = self
            .files
            .iter()
            .enumerate()
            .filter_map(|(index, download)| match download.state {
                DownloadState::Waiting => self.waiting_download(index),
                _ => None,
            })
            .fold(content, |acc, x| acc.push(x));
        let content = scrollable(content.spacing(3.));
        Some(content.into())
    }

    fn waiting_download(&self, index: usize) -> Option<row::Row<'_, Message>> {
        let txt = Text::new(format!("=> {:#?}", self.files[index].path));
        let priorty = self.waiting_download_priority(index);
        let content = row![txt, priorty].align_y(Alignment::Center).spacing(3.);
        Some(content)
    }

    fn waiting_download_priority(&self, index: usize) -> Column<'_, Message> {
        let up = svg_button(IconName::Up.get()).on_press_maybe(
            self.is_upgradable(index)
                .then_some(DownloadMessage::UpgradePriorty(index).into()),
        );
        let down = svg_button(IconName::Down.get()).on_press_maybe(
            self.is_downgradable(index)
                .then_some(DownloadMessage::DowngradePriorty(index).into()),
        );
        column![up, down].align_x(Alignment::Center).spacing(2.)
    }

    fn finished_view(&self) -> Option<Element<'_, Message>> {
        if self.finished_count == 0 {
            return None;
        }
        let title = Text::new("finished downloads");
        let content = column![title];
        let content = self
            .files
            .iter()
            .filter_map(|download| match download.state {
                DownloadState::Finished => {
                    let txt = Text::new(format!("=> {:#?}", download.path));
                    Some(txt)
                }
                _ => None,
            })
            .fold(content, |acc, x| acc.push(x));
        let content = scrollable(content.spacing(3.));
        Some(content.into())
    }
    fn failed_view(&self) -> Option<Element<'_, Message>> {
        if self.failed_count == 0 {
            return None;
        }
        let title = Text::new("failed downloads");
        let content = column![title];
        let content = self
            .files
            .iter()
            .enumerate()
            .filter_map(|(index, download)| match &download.state {
                DownloadState::Failed(err) => {
                    let txt = Text::new(format!("=> {:#?} because of {:#?}", download.path, err));
                    let retry_btn = svg_button(IconName::Retry.get())
                        .on_press(DownloadMessage::RetryFailed(index).into());
                    Some(row![txt, retry_btn])
                }
                _ => None,
            })
            .fold(content, |acc, x| acc.push(x));
        let content = scrollable(content.spacing(3.));
        Some(content.into())
    }
    fn canceled_view(&self) -> Option<Element<'_, Message>> {
        if self.canceled_count == 0 {
            return None;
        }
        let title = Text::new("canceled downloads");
        let content = column![title];
        let content = self
            .files
            .iter()
            .enumerate()
            .filter_map(|(index, download)| match download.state {
                DownloadState::Canceled => {
                    let txt = Text::new(format!("=> {:#?}", download.path));
                    let retry_btn = svg_button(IconName::Retry.get())
                        .on_press(DownloadMessage::RetryCanceled(index).into());
                    Some(row![txt, retry_btn])
                }
                _ => None,
            })
            .fold(content, |acc, x| acc.push(x));
        let content = scrollable(content.spacing(3.));
        Some(content.into())
    }
}

async fn get_download_paths(grpc: RpcClient, units: Vec<Unit>) -> Result<Vec<PathBuf>, RpcError> {
    let mut res = Vec::new();
    for unit in units {
        match unit.kind {
            UnitKind::Folder => {
                let in_units = grpc.clone().ls(unit.path.clone()).await?;
                let mut folders = Vec::new();
                for in_unit in in_units {
                    match in_unit.kind {
                        UnitKind::Folder => {
                            folders.push(in_unit);
                        }
                        _ => {
                            res.push(in_unit.path.clone());
                        }
                    }
                }
                res.extend(Box::pin(get_download_paths(grpc.clone(), folders)).await?);
            }
            _ => {
                res.push(unit.path.clone());
            }
        };
    }
    Ok(res)
}

#[derive(Clone)]
pub enum DownloadProgress {
    Begin {
        index: usize,
        total_size: u64,
    },
    Progressed {
        index: usize,
        by: usize,
    },
    Finish(usize),
    Close {
        index: usize,
        result: Result<(), RpcError>,
    },
}

fn join_downloads(path: &Path) -> PathBuf {
    home_dir().unwrap().join("Downloads").join(path)
}

fn download_file(
    grpc: RpcClient,
    index: usize,
    target: PathBuf,
) -> impl Straw<(), DownloadProgress, RpcError> {
    sipper(async move |mut sender| {
        let (size, mut stream) = grpc.download_stream(&target).await?;
        sender
            .send(DownloadProgress::Begin {
                index,
                total_size: size,
            })
            .await;

        let target = join_downloads(&target);
        create_dir_all(target.parent().map(|x| x.to_path_buf()).unwrap_or_default()).await?;
        let _ = remove_file(&target).await;
        let mut file = File::create(&target).await?;
        loop {
            match stream.message().await {
                Ok(dr) => {
                    match dr {
                        Some(DownloadResponse { data }) => {
                            file.write_all(&data).await?;
                            file.flush().await?;
                            sender
                                .send(DownloadProgress::Progressed {
                                    index,
                                    by: data.len(),
                                })
                                .await;
                        }
                        None => {
                            sender.send(DownloadProgress::Finish(index)).await;
                            return Ok(());
                        }
                    };
                }
                Err(status) => {
                    remove_file(&target).await?;
                    return Err(RpcError::TonicStatus(status));
                }
            }
        }
    })
}
