use assets::{CLOSE_SVG, IconData, SELECT_SVG};
use async_recursion::async_recursion;
use common::{Origin, Selected, Unit, UnitKind};
use delivery::{Delivery, download_file};
use iced::{
    Border, Color, Length, Task,
    theme::Palette,
    widget::{
        Button, Column, Container, Svg, Text, button::Style, column, row, scrollable, svg::Handle,
    },
    window,
};
use std::{
    env::home_dir,
    net::{IpAddr, Ipv4Addr},
    path::{Path, PathBuf},
};

use crate::{Message, home::go_home_button};

#[derive(Debug, Clone)]
pub enum ClientMessage {
    ChangeCurrentPath(PathBuf),
    CurrentPathChanged {
        current_path: PathBuf,
        units: Vec<Unit>,
    },
    GoBack,
    GoneBack(Vec<Unit>),
    ToggleSelectMode,
    Select(Unit),
    OpenDownloadWindow,
    DownloadWindowOpened(window::Id),
    CloseDownloadWindow(window::Id),
    DownloadWindowClosed,
    PrepareDownloading,
    StartDownloading(Vec<Download>),
    DownloadDone,
}

impl ClientMessage {
    pub fn handle(self, state: &mut ClientState) -> Task<Message> {
        match self {
            ClientMessage::ChangeCurrentPath(path_buf) => {
                Task::perform(state.delivery.clone().ls(path_buf.clone()), move |xs| {
                    if let Ok(xs) = xs {
                        Message::Client(ClientMessage::CurrentPathChanged {
                            units: xs,
                            current_path: path_buf.clone(),
                        })
                    } else {
                        Message::None
                    }
                })
            }
            ClientMessage::CurrentPathChanged {
                current_path,
                units,
            } => {
                state.units = units;
                state.current_path = current_path;
                Task::none()
            }
            ClientMessage::GoBack => {
                if let Some(parent) = state.current_path.parent() {
                    Task::perform(state.delivery.clone().ls(parent.to_path_buf()), |xs| {
                        if let Ok(xs) = xs {
                            Message::Client(ClientMessage::GoneBack(xs))
                        } else {
                            Message::None
                        }
                    })
                } else {
                    Task::none()
                }
            }
            ClientMessage::GoneBack(units) => {
                state.current_path.pop();
                state.units = units;
                Task::none()
            }
            ClientMessage::ToggleSelectMode => {
                if state.select.on {
                    state.select.clear();
                } else {
                    state.select.on = true;
                }
                Task::none()
            }
            ClientMessage::Select(unit) => {
                state.select.toggle_unit_selection(&unit);
                Task::none()
            }
            ClientMessage::PrepareDownloading => {
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
                            Message::Client(ClientMessage::StartDownloading(downloads))
                        }
                        Err(err) => Message::ErrorHappned(format!(
                            "error : {err:#?} happened\n while preparing for downloading"
                        )),
                    },
                )
            }
            ClientMessage::StartDownloading(download_tasks) => {
                let origin = state.delivery.origin.clone();
                let (tasks, handles): (Vec<_>, Vec<_>) = download_tasks
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
                let (task, handle) = Task::batch(tasks).abortable();
                state.downloads.state = DownloadingState::Downloading {
                    main_handle: handle,
                    tasks_handles: handles,
                    tasks: download_tasks,
                };
                task.map(|x| match x {
                    Ok(_) => Message::Client(ClientMessage::DownloadDone),
                    Err(err) => Message::ErrorHappned(format!(
                        "error : {err:#?} happened \nwhile downloading"
                    )),
                })
            }
            ClientMessage::DownloadDone => {
                state.downloads.clear();
                Task::none()
            }
            ClientMessage::OpenDownloadWindow => {
                let (_, task) = window::open(window::Settings::default());
                task.map(|id| Message::Client(ClientMessage::DownloadWindowOpened(id)))
            }
            ClientMessage::DownloadWindowOpened(id) => {
                state.download_window = Some(id);
                Task::none()
            }
            ClientMessage::CloseDownloadWindow(id) => {
                let task = window::close(id);
                task.map(|_: window::Id| Message::Client(ClientMessage::DownloadWindowClosed))
            }
            ClientMessage::DownloadWindowClosed => {
                state.download_window = None;
                Task::none()
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Download {
    server_path: PathBuf,
    host_path: PathBuf,
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

pub struct ClientState {
    pub delivery: Delivery,
    pub units: Vec<Unit>,
    pub current_path: PathBuf,
    pub select: Selected,
    pub download_window: Option<window::Id>,
    pub downloads: Downloads,
}

pub struct Downloads {
    state: DownloadingState,
    download_dir: PathBuf,
}

impl Downloads {
    pub fn view(&self) -> Container<'_, Message> {
        Container::new(Text::new("downloads"))
    }
}

enum DownloadingState {
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
    fn clear(&mut self) {
        self.state = DownloadingState::Done;
    }
}

impl Default for ClientState {
    fn default() -> Self {
        Self {
            delivery: Delivery::new(
                Origin {
                    ip: IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
                    port: 0,
                }
                .to_string(),
            ),
            units: Vec::new(),
            current_path: PathBuf::new(),
            select: Selected::default(),
            downloads: Downloads::new(),
            download_window: None,
        }
    }
}

impl ClientState {
    pub fn view(&self, window_id: window::Id) -> Container<'_, Message> {
        if self.download_window.is_some_and(|x| x == window_id) {
            return self.downloads.view();
        }
        self.main_window()
    }

    fn main_window(&self) -> Container<'_, Message> {
        let tools = self.tools_bar();
        let units = self
            .units
            .iter()
            .fold(Column::new().spacing(10.), |acc, x| {
                acc.push(x.button(&self.select))
            });
        let units = scrollable(units).width(Length::Fill);
        let all = column![tools, units].spacing(14.).width(Length::Fill);
        Container::new(all)
    }

    fn tools_bar(&self) -> Column<'_, Message> {
        let home = go_home_button();
        let back = Button::new("back").on_press(Message::Client(ClientMessage::GoBack));
        let selector = Button::new(svg_from_icon_data(if self.select.on {
            &CLOSE_SVG
        } else {
            &SELECT_SVG
        }))
        .on_press(Message::Client(ClientMessage::ToggleSelectMode));
        let download = Button::new(match self.download_window {
            Some(_) => "close download window",
            None => "open download window",
        })
        .on_press(match self.download_window {
            Some(id) => Message::Client(ClientMessage::CloseDownloadWindow(id)),
            None => Message::Client(ClientMessage::OpenDownloadWindow),
        });
        column![selector, back, home, download].spacing(5.)
    }
}

trait UnitViews {
    fn button<'a>(&'a self, selected: &'a Selected) -> Button<'a, Message>;
}

impl UnitViews for Unit {
    fn button<'a>(&'a self, selected: &'a Selected) -> Button<'a, Message> {
        let svg = svg_from_icon_data(self.icon());
        let text = Text::new(self.name());
        let row = row![svg, text].spacing(4.);
        Button::new(row)
            .on_press({
                let message = if selected.on {
                    ClientMessage::Select(self.clone())
                } else {
                    ClientMessage::ChangeCurrentPath(self.path.clone())
                };
                Message::Client(message)
            })
            .style(|theme, _| {
                let selected = selected.is_selected(self);
                let Palette {
                    text,
                    background,
                    danger,
                    ..
                } = theme.palette();
                Style {
                    border: Border {
                        color: if selected { danger } else { Color::BLACK },
                        width: if selected { 5. } else { 0. },
                        ..Default::default()
                    },
                    text_color: text,
                    background: Some(iced::Background::Color(background)),
                    ..Default::default()
                }
            })
    }
}

pub fn svg_from_icon_data(icon: &IconData) -> Svg<'_> {
    let handle = Handle::from_memory(icon.data.bytes().collect::<Vec<_>>());
    Svg::new(handle).width(30.)
}
