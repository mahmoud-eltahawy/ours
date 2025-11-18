use crate::{
    Page,
    client::{self, svg_button},
};
use common::assets::IconName;
use grpc::{
    UnitKind,
    client::{DownloadResponse, ResumeDownloadResponse, RpcClient},
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
    fs::{File, OpenOptions, create_dir_all, remove_file},
    io::AsyncWriteExt,
};

#[derive(Default, Debug, Clone)]
pub struct Downloads {
    pub show_preview: bool,
    progressing: Vec<(usize, Handle)>,
    waiting: Vec<usize>,
    resumable: Vec<usize>,
    finished: Vec<usize>,
    paused: Vec<usize>,
    failed: Vec<(usize, RpcError)>,
    canceled: Vec<usize>,
    files: Vec<Download>,
}

#[derive(Clone)]
pub enum Message {
    TogglePreview,
    QueueFromSelectedStart,
    QueueFromSelected(Result<Vec<PathBuf>, RpcError>),
    Tick(DownloadProgress),
    CancelProgress(usize, Handle),
    Pause(usize, Handle),
    Resume(usize),
    ProgressCanceled(usize),
    UpgradePriorty(usize),
    DowngradePriorty(usize),
    CanceledToWait(usize),
    RetryFailed(usize),
}

impl From<Message> for crate::Message {
    fn from(value: Message) -> Self {
        crate::Message::Client(client::Message::Download(value))
    }
}

impl crate::State {
    pub fn handle_downloads_msg(&mut self, msg: Message, grpc: RpcClient) -> Task<crate::Message> {
        let Page::Client(state) = &mut self.page else {
            unreachable!()
        };
        match msg {
            Message::QueueFromSelectedStart => {
                let units = state.select.units.clone();
                state.select.clear();
                let fut = get_download_paths(grpc.clone(), units);
                Task::perform(fut, |x| Message::QueueFromSelected(x).into())
            }
            Message::QueueFromSelected(paths) => {
                let paths = match paths {
                    Ok(paths) => paths,
                    Err(err) => {
                        dbg!(err);
                        return Task::none();
                    }
                };
                self.downloads.waitlist_extend(paths);
                self.downloads.tick_available(grpc)
            }
            Message::Tick(download_progress) => match download_progress {
                DownloadProgress::Begin { index, total_size } => {
                    self.downloads.files[index].total_size = total_size as usize;
                    Task::none()
                }
                DownloadProgress::Progressed { index, by } => {
                    self.downloads.files[index].sended += by;
                    Task::none()
                }
                DownloadProgress::Finish(index) => {
                    self.downloads.finish_list(index);
                    Task::none()
                }
                DownloadProgress::CheckDownloadResult { index, result } => {
                    if let Err(err) = result {
                        self.downloads.progress_fail_list(index, err);
                    }
                    self.downloads.tick_available(grpc)
                }
            },
            Message::TogglePreview => {
                self.downloads.show_preview = !self.downloads.show_preview;
                Task::none()
            }
            Message::CancelProgress(index, handle) => {
                handle.abort();
                Task::perform(
                    remove_file(join_downloads(&self.downloads.files[index].path)),
                    move |_| Message::ProgressCanceled(index).into(),
                )
            }
            Message::Pause(index, handle) => {
                handle.abort();
                self.downloads.pause_list(index);
                self.downloads.tick_available(grpc)
            }
            Message::Resume(index) => {
                self.downloads.resume_list(index);
                self.downloads.tick_available(grpc)
            }
            Message::ProgressCanceled(index) => {
                self.downloads.progress_cancel_list(index);
                self.downloads.tick_available(grpc)
            }
            Message::UpgradePriorty(index) => {
                self.downloads.upgrade_waiting(index);
                Task::none()
            }
            Message::DowngradePriorty(index) => {
                self.downloads.downgrade_waiting(index);
                Task::none()
            }
            Message::CanceledToWait(index) => {
                self.downloads.waiting_cancel_list(index);
                self.downloads.tick_available(grpc)
            }
            Message::RetryFailed(index) => {
                self.downloads.fail_wait_list(index);
                self.downloads.tick_available(grpc)
            }
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct Download {
    path: PathBuf,
    total_size: usize,
    sended: usize,
}

impl From<PathBuf> for Download {
    fn from(value: PathBuf) -> Self {
        Self {
            path: value,
            total_size: 0,
            sended: 0,
        }
    }
}

enum Turn {
    Waiting(usize),
    Resumable(usize),
}

impl Downloads {
    fn waitlist_extend(&mut self, paths: Vec<PathBuf>) {
        let before_len = self.files.len();
        self.files.extend(paths.into_iter().map(Download::from));
        let after_len = self.files.len();
        self.waiting.extend(before_len..after_len);
    }
    pub fn active_count(&self) -> usize {
        self.progressing.len() + self.waiting.len() + self.resumable.len()
    }
    fn is_upgradable(&self, index: usize) -> bool {
        self.waiting.contains(&index) && self.waiting[0] != index
    }
    fn is_downgradable(&self, index: usize) -> bool {
        self.waiting.contains(&index) && self.waiting[self.waiting.len() - 1] != index
    }
    fn upgrade_waiting(&mut self, index: usize) {
        if !self.is_upgradable(index) {
            return;
        }
        let index_index = self.waiting.iter().position(|x| *x == index).unwrap();
        let target_index = index_index - 1;
        let target = self.waiting[target_index];
        self.waiting[index_index] = target;
        self.waiting[target_index] = index;
    }
    fn downgrade_waiting(&mut self, index: usize) {
        if !self.is_downgradable(index) {
            return;
        }
        let index_index = self.waiting.iter().position(|x| *x == index).unwrap();
        let target_index = index_index + 1;
        let target = self.waiting[target_index];
        self.waiting[index_index] = target;
        self.waiting[target_index] = index;
    }
    fn wait_progress_list(&mut self, index: usize, handle: Handle) {
        self.waiting.retain(|x| *x != index);
        self.progressing.push((index, handle));
    }
    fn resumable_progress_list(&mut self, index: usize, handle: Handle) {
        self.resumable.retain(|x| *x != index);
        self.progressing.push((index, handle));
    }
    fn finish_list(&mut self, index: usize) {
        self.progressing.retain(|x| x.0 != index);
        self.finished.push(index);
    }

    fn pause_list(&mut self, index: usize) {
        self.progressing.retain(|x| x.0 != index);
        self.paused.push(index);
    }

    fn resume_list(&mut self, index: usize) {
        self.resumable.push(index);
        self.paused.retain(|x| *x != index);
    }

    fn progress_fail_list(&mut self, index: usize, err: RpcError) {
        self.progressing.retain(|x| x.0 != index);
        self.failed.push((index, err));
        self.files[index].sended = 0;
    }

    fn waiting_cancel_list(&mut self, index: usize) {
        self.canceled.retain(|x| *x != index);
        self.waiting.push(index);
    }

    fn fail_wait_list(&mut self, index: usize) {
        self.failed.retain(|x| x.0 != index);
        self.waiting.push(index);
    }

    fn progress_cancel_list(&mut self, index: usize) {
        self.progressing.retain(|x| x.0 != index);
        self.canceled.push(index);
        self.files[index].sended = 0;
    }
    fn next_turn(&self) -> Option<Turn> {
        if let Some(index) = self.resumable.first() {
            Some(Turn::Resumable(*index))
        } else {
            self.waiting.first().map(|index| Turn::Waiting(*index))
        }
    }
    fn turn_task(&mut self, grpc: RpcClient) -> Option<Task<DownloadProgress>> {
        if self.progressing.len() >= 5 {
            return None;
        }
        match self.next_turn()? {
            Turn::Waiting(index) => {
                let download = &self.files[index];
                let (task, handle) = Task::sip(
                    download_file(grpc.clone(), index, download.path.clone()),
                    |progress| progress,
                    move |x| DownloadProgress::CheckDownloadResult { index, result: x },
                )
                .abortable();

                self.wait_progress_list(index, handle);
                Some(task)
            }
            Turn::Resumable(index) => {
                let download = &self.files[index];
                let (task, handle) = Task::sip(
                    resume_file(grpc.clone(), index, download.sended, download.path.clone()),
                    |progress| progress,
                    move |x| DownloadProgress::CheckDownloadResult { index, result: x },
                )
                .abortable();
                self.resumable_progress_list(index, handle);
                Some(task)
            }
        }
    }

    fn tick_available(&mut self, grpc: RpcClient) -> Task<crate::Message> {
        let mut xs = Vec::new();

        while let Some(task) = self
            .turn_task(grpc.clone())
            .map(|task| task.map(move |download_progress| Message::Tick(download_progress).into()))
        {
            xs.push(task);
        }
        Task::batch(xs)
    }

    pub fn view(&self) -> Element<'_, crate::Message> {
        let title = Text::new("Downloads");
        let progressing = self.progressing_view();
        let waiting = self.waiting_view();
        let failed = self.failed_view();
        let finished = self.finished_view();
        let paused = self.paused_view();
        let canceled = self.canceled_view();
        let content = scrollable(
            column![
                title,
                progressing,
                waiting,
                failed,
                finished,
                paused,
                canceled
            ]
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

    fn progressing_view(&self) -> Option<Element<'_, crate::Message>> {
        if self.progressing.is_empty() {
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
            .progressing
            .iter()
            .map(|(index, handle)| {
                let index = *index;
                let download = &self.files[index];
                let txt = Text::new(format!(
                    "{}, {} of {},{:.2}%",
                    download.path.display(),
                    format_size(download.sended),
                    format_size(download.total_size),
                    (download.sended as f32 / download.total_size as f32) * 100.0
                ));
                let progress_bar =
                    progress_bar(0.0..=(download.total_size as f32), download.sended as f32);
                let left = column![txt, progress_bar].align_x(Alignment::Center);
                let cancel = svg_button(IconName::Close.get())
                    .height(Length::Fixed(80.))
                    .clip(false)
                    .on_press(Message::CancelProgress(index, handle.clone()).into());
                let pause = svg_button(IconName::Pause.get())
                    .height(Length::Fixed(80.))
                    .clip(false)
                    .on_press(Message::Pause(index, handle.clone()).into());
                let buttons = column![cancel, pause].spacing(3.);
                row![left, buttons].align_y(Alignment::Center).spacing(5.)
            })
            .fold(content, |acc, x| acc.push(x));
        let content = scrollable(content.spacing(3.));
        Some(content.into())
    }

    fn waiting_view(&self) -> Option<Element<'_, crate::Message>> {
        if self.waiting.is_empty() {
            return None;
        }
        let title = Text::new("waiting downloads");
        let content = column![title];
        let content = match self.resumables() {
            Some(rs) => content.push(rs),
            None => content,
        };
        let content = self
            .waiting
            .iter()
            .map(|index| self.waiting_download(*index))
            .fold(content, |acc, x| acc.push(x));
        let content = scrollable(content.spacing(3.));
        Some(content.into())
    }

    fn resumables(&self) -> Option<Column<'_, crate::Message>> {
        if self.resumable.is_empty() {
            return None;
        }
        let title = Text::new("resumable downloads");
        let content = column![title];
        let content = self
            .resumable
            .iter()
            .map(|index| Text::new(format!("=> {:#?}", self.files[*index].path)))
            .fold(content, |acc, x| acc.push(x));
        Some(content)
    }

    fn waiting_download(&self, index: usize) -> row::Row<'_, crate::Message> {
        let txt = Text::new(format!("=> {:#?}", self.files[index].path));
        let priorty = self.waiting_download_priority(index);
        row![txt, priorty].align_y(Alignment::Center).spacing(3.)
    }

    fn waiting_download_priority(&self, index: usize) -> Column<'_, crate::Message> {
        let up = svg_button(IconName::Up.get()).on_press_maybe(
            self.is_upgradable(index)
                .then_some(Message::UpgradePriorty(index).into()),
        );
        let down = svg_button(IconName::Down.get()).on_press_maybe(
            self.is_downgradable(index)
                .then_some(Message::DowngradePriorty(index).into()),
        );
        column![up, down].align_x(Alignment::Center).spacing(2.)
    }

    fn finished_view(&self) -> Option<Element<'_, crate::Message>> {
        if self.finished.is_empty() {
            return None;
        }
        let title = Text::new("finished downloads");
        let content = column![title];
        let content = self
            .finished
            .iter()
            .map(|index| {
                let download = &self.files[*index];
                Text::new(format!("=> {}", download.path.display()))
            })
            .fold(content, |acc, x| acc.push(x));
        let content = scrollable(content.spacing(3.));
        Some(content.into())
    }
    fn failed_view(&self) -> Option<Element<'_, crate::Message>> {
        if self.failed.is_empty() {
            return None;
        }
        let title = Text::new("failed downloads");
        let content = column![title];
        let content = self
            .failed
            .iter()
            .map(|(index, err)| {
                let download = &self.files[*index];
                let txt = Text::new(format!(
                    "=> {} because of {:#?}",
                    download.path.display(),
                    err
                ));
                let retry_btn =
                    svg_button(IconName::Retry.get()).on_press(Message::RetryFailed(*index).into());
                row![txt, retry_btn]
            })
            .fold(content, |acc, x| acc.push(x));
        let content = scrollable(content.spacing(3.));
        Some(content.into())
    }

    fn paused_view(&self) -> Option<Element<'_, crate::Message>> {
        if self.paused.is_empty() {
            return None;
        }
        let title = Text::new("paused downloads");
        let content = column![title];
        let content = self
            .paused
            .iter()
            .map(|index| {
                let download = &self.files[*index];
                let txt = Text::new(format!("=> {}", download.path.display()));
                let retry_btn =
                    svg_button(IconName::Retry.get()).on_press(Message::Resume(*index).into());
                row![txt, retry_btn]
            })
            .fold(content, |acc, x| acc.push(x));

        let content = scrollable(content.spacing(3.));
        Some(content.into())
    }

    fn canceled_view(&self) -> Option<Element<'_, crate::Message>> {
        if self.canceled.is_empty() {
            return None;
        }
        let title = Text::new("canceled downloads");
        let content = column![title];
        let content = self
            .canceled
            .iter()
            .map(|index| {
                let download = &self.files[*index];
                let txt = Text::new(format!("=> {}", download.path.display()));
                let retry_btn = svg_button(IconName::Retry.get())
                    .on_press(Message::CanceledToWait(*index).into());
                row![txt, retry_btn]
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
    CheckDownloadResult {
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

fn resume_file(
    grpc: RpcClient,
    index: usize,
    progress_index: usize,
    target: PathBuf,
) -> impl Straw<(), DownloadProgress, RpcError> {
    sipper(async move |mut sender| {
        let mut stream = grpc.resume_stream(progress_index, &target).await?;
        let target = join_downloads(&target);
        let mut file = OpenOptions::new().append(true).open(&target).await?;
        loop {
            match stream.message().await {
                Ok(dr) => {
                    match dr {
                        Some(ResumeDownloadResponse { data }) => {
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
