use crate::{Message, State, client::svg_button};
use common::assets::IconName;
use grpc::{UnitKind, client::RpcClient, error::RpcError, top::Unit};
use iced::{
    Alignment, Background, Border, Element, Length, Task, Theme,
    border::Radius,
    task::Handle,
    widget::{Button, Column, Container, Text, column, container, row, scrollable},
};
use std::path::PathBuf;

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
    TickNext {
        result: Result<(), RpcError>,
        index: usize,
    },
    CancelProgress(usize, Handle),
    UpgradePriorty(usize),
    DowngradePriorty(usize),
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
            DownloadMessage::TickNext { result, index } => {
                match result {
                    Ok(()) => {
                        state.downloads.finish(index);
                    }
                    Err(err) => {
                        dbg!(&err);
                        state.downloads.fail(index, err);
                    }
                }
                let Some(grpc) = state.grpc.clone() else {
                    return Task::none();
                };
                state.downloads.tick_available(grpc)
            }
            DownloadMessage::TogglePreview => {
                state.downloads.show_preview = !state.downloads.show_preview;
                Task::none()
            }
            DownloadMessage::CancelProgress(index, handle) => {
                handle.abort();
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
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct Download {
    path: PathBuf,
    state: DownloadState,
}

impl From<PathBuf> for Download {
    fn from(value: PathBuf) -> Self {
        Self {
            path: value,
            state: DownloadState::Waiting,
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
        self.finished_count += 1;
        self.files[index].state = DownloadState::Failed(err);
    }
    fn cancel(&mut self, index: usize) {
        self.progressing_count -= 1;
        self.canceled_count += 1;
        self.files[index].state = DownloadState::Canceled;
    }
    fn first_waiting(&mut self) -> Option<(&Download, usize)> {
        for (i, value) in self.files.iter().enumerate() {
            if matches!(value.state, DownloadState::Waiting) {
                return Some((value, i));
            }
        }
        None
    }
    fn fill_turn(&mut self, grpc: RpcClient) -> Option<(Task<Result<(), RpcError>>, usize)> {
        if self.progressing_count >= 5 {
            return None;
        }
        match self.first_waiting() {
            Some((download, index)) => {
                let (task, handle) =
                    Task::future(grpc.clone().download_file(download.path.clone())).abortable();
                self.progress(index, handle);
                Some((task, index))
            }
            None => None,
        }
    }

    fn tick_next(&mut self, grpc: RpcClient) -> Option<Task<Message>> {
        self.fill_turn(grpc).map(|(task, index)| {
            task.map(move |result| DownloadMessage::TickNext { result, index }.into())
        })
    }
    fn tick_available(&mut self, grpc: RpcClient) -> Task<Message> {
        let mut xs = Vec::new();

        while let Some(task) = self.tick_next(grpc.clone()) {
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
                .width(Length::Shrink)
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

        let title = Text::new("in progress downloads");
        let content = column![title];
        let content = self
            .files
            .iter()
            .enumerate()
            .filter_map(|(index, download)| match &download.state {
                DownloadState::Progressing(handle) => {
                    let txt = Text::new(format!("=> {:#?}", download.path));
                    let button = Button::new(svg_button(IconName::Close.get()))
                        .on_press(DownloadMessage::CancelProgress(index, handle.clone()).into());
                    let row = row![txt, button].align_y(Alignment::Center).spacing(5.);
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
        let up = Button::new(svg_button(IconName::Up.get())).on_press_maybe(
            self.is_upgradable(index)
                .then_some(DownloadMessage::UpgradePriorty(index).into()),
        );
        let down = Button::new(svg_button(IconName::Down.get())).on_press_maybe(
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
            .filter_map(|download| match &download.state {
                DownloadState::Failed(err) => {
                    let txt = Text::new(format!("=> {:#?} because of {:#?}", download.path, err));
                    Some(txt)
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
            .filter_map(|download| match download.state {
                DownloadState::Canceled => {
                    let txt = Text::new(format!("=> {:#?}", download.path));
                    Some(txt)
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
