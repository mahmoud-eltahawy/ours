use crate::{Message, client::ClientMessage};
use grpc::{client::RpcClient, error::RpcError};
use iced::{Task, task::Handle};
use std::path::PathBuf;

#[derive(Default, Debug)]
pub struct Downloads {
    pub progressing_count: usize,
    pub waiting_count: usize,
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
        self.files[index].state = DownloadState::Finished;
    }
    pub fn fail(&mut self, index: usize, err: RpcError) {
        self.progressing_count -= 1;
        self.files[index].state = DownloadState::Failed(err);
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
}
