use crate::{Message, State};
use grpc::{UnitKind, client::RpcClient, error::RpcError, top::Unit};
use iced::{
    Background, Border, Element, Task, Theme,
    border::Radius,
    task::Handle,
    widget::{Button, Container, Text, column, container, row, scrollable},
};
use std::path::PathBuf;

#[derive(Default, Debug)]
pub struct Downloads {
    pub show_preview: bool,
    pub progressing_count: usize,
    pub waiting_count: usize,
    pub finished_count: usize,
    pub failed_count: usize,
    pub canceled_count: usize,
    pub files: Vec<Download>,
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
        }
    }
}

#[derive(Default, Debug)]
pub struct Download {
    pub path: PathBuf,
    pub state: DownloadState,
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
    pub fn wait(&mut self, paths: Vec<PathBuf>) {
        self.waiting_count += paths.len();
        self.files.extend(paths.into_iter().map(Download::from));
    }
    pub fn active_count(&self) -> usize {
        self.progressing_count + self.waiting_count
    }
    pub fn progress(&mut self, index: usize, handle: Handle) {
        self.waiting_count -= 1;
        self.progressing_count += 1;
        self.files[index].state = DownloadState::Progressing(handle);
    }
    pub fn finish(&mut self, index: usize) {
        self.progressing_count -= 1;
        self.finished_count += 1;
        self.files[index].state = DownloadState::Finished;
    }
    pub fn fail(&mut self, index: usize, err: RpcError) {
        self.progressing_count -= 1;
        self.finished_count += 1;
        self.files[index].state = DownloadState::Failed(err);
    }
    pub fn cancel(&mut self, index: usize) {
        self.progressing_count -= 1;
        self.canceled_count += 1;
        self.files[index].state = DownloadState::Canceled;
    }
    pub fn first_waiting(&mut self) -> Option<(&Download, usize)> {
        for (i, value) in self.files.iter().enumerate() {
            if matches!(value.state, DownloadState::Waiting) {
                return Some((value, i));
            }
        }
        None
    }
    pub fn fill_turn(&mut self, grpc: RpcClient) -> Option<(Task<Result<(), RpcError>>, usize)> {
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

    pub fn tick_next(&mut self, grpc: RpcClient) -> Option<Task<Message>> {
        self.fill_turn(grpc).map(|(task, index)| {
            task.map(move |result| DownloadMessage::TickNext { result, index }.into())
        })
    }
    pub fn tick_available(&mut self, grpc: RpcClient) -> Task<Message> {
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
            column![title, progressing, waiting, failed, finished, canceled].spacing(20.),
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
            .into()
    }

    pub fn progressing_view(&self) -> Option<Element<'_, Message>> {
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
                    let button = Button::new("cancel")
                        .on_press(DownloadMessage::CancelProgress(index, handle.clone()).into());
                    let row = row![txt, button].spacing(5.);
                    Some(row)
                }
                _ => None,
            })
            .fold(content, |acc, x| acc.push(x));
        let content = scrollable(content.spacing(3.));
        Some(content.into())
    }

    pub fn waiting_view(&self) -> Option<Element<'_, Message>> {
        if self.waiting_count == 0 {
            return None;
        }
        let title = Text::new("waiting downloads");
        let content = column![title];
        let content = self
            .files
            .iter()
            .filter_map(|download| match download.state {
                DownloadState::Waiting => {
                    let txt = Text::new(format!("=> {:#?}", download.path));
                    Some(txt)
                }
                _ => None,
            })
            .fold(content, |acc, x| acc.push(x));
        let content = scrollable(content.spacing(3.));
        Some(content.into())
    }
    pub fn failed_view(&self) -> Option<Element<'_, Message>> {
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
    pub fn finished_view(&self) -> Option<Element<'_, Message>> {
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
    pub fn canceled_view(&self) -> Option<Element<'_, Message>> {
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
