use crate::{Message, client::ClientMessage};
use grpc::{client::RpcClient, error::RpcError};
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
            task.map(move |result| ClientMessage::TickNextDownload { result, index }.into())
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
                    let button = Button::new("cancel").on_press(
                        ClientMessage::CancelDownloadProgress(index, handle.clone()).into(),
                    );
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
        let content = Text::new("failed");
        Some(content.into())
    }
    pub fn finished_view(&self) -> Option<Element<'_, Message>> {
        if self.finished_count == 0 {
            return None;
        }
        let content = Text::new("finished");
        Some(content.into())
    }
    pub fn canceled_view(&self) -> Option<Element<'_, Message>> {
        if self.canceled_count == 0 {
            return None;
        }
        let content = Text::new("canceled");
        Some(content.into())
    }
}
