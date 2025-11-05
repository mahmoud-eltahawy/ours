use crate::home::go_home_button;
use crate::{Message, Page, State, svg_from_icon_data};
use common::assets::IconName;
use grpc::UnitKind;
use grpc::client::RpcClient;
use grpc::error::RpcError;
use grpc::top::{Selected, Unit};
use iced::Task;
use iced::task::Handle;
use iced::theme::Palette;
use iced::widget::container;
use iced::{
    Border, Element, Length,
    border::Radius,
    mouse::Interaction,
    widget::{Button, Container, MouseArea, Row, Text, button::Style, mouse_area, row, scrollable},
};
use std::path::PathBuf;

#[derive(Default)]
pub struct ClientState {
    pub grpc: Option<RpcClient>,
    pub target: PathBuf,
    pub select: Selected,
    pub units: Vec<Unit>,
    downloads: Downloads,
}

#[derive(Default, Debug)]
struct Downloads {
    pub waiting: Vec<PathBuf>,
    pub progressing: Vec<Option<(PathBuf, Handle)>>,
    pub finished: Vec<PathBuf>,
    pub failed: Vec<(PathBuf, RpcError)>,
}

impl Downloads {
    fn fill_turn(&mut self, grpc: RpcClient) -> Option<(Task<Result<(), RpcError>>, usize)> {
        match self.waiting.pop() {
            Some(path) => {
                let (t, h) = Task::future(grpc.clone().download_file(path.clone())).abortable();
                self.progressing.push(Some((path, h)));
                Some((t, self.progressing.len() - 1))
            }
            None => None,
        }
    }

    fn tick_next(&mut self, grpc: RpcClient) -> Option<Task<Message>> {
        self.fill_turn(grpc).map(|(task, index)| {
            task.map(move |result| ClientMessage::TickNextDownload { result, index }.into())
        })
    }
    fn tick_available(&mut self, grpc: RpcClient) -> Task<Message> {
        let mut xs = Vec::new();

        while let Some(task) = self.tick_next(grpc.clone()) {
            xs.push(task);
        }
        Task::batch(xs)
    }
    fn fail(&mut self, index: usize, err: RpcError) {
        let target = self.progressing[index].clone().map(|x| x.0);
        if let Some(target) = target {
            self.failed.push((target, err));
        }
        self.progressing[index] = None;
    }
    fn finish(&mut self, index: usize) {
        let target = self.progressing[index].clone().map(|x| x.0);
        if let Some(target) = target {
            self.finished.push(target);
        }
        self.progressing[index] = None;
    }
}

impl ClientState {
    pub fn new(grpc: RpcClient) -> Self {
        Self {
            grpc: Some(grpc),
            target: PathBuf::new(),
            units: Vec::new(),
            select: Selected::default(),
            downloads: Downloads::default(),
        }
    }
}

#[derive(Clone)]
pub enum ClientMessage {
    RefreshUnits(Result<Vec<Unit>, RpcError>),
    PrepareGrpc(Result<RpcClient, RpcError>),
    UnitClick(Unit),
    UnitDoubleClick(Unit),
    QueueDownloadFromSelectedStart,
    QueueDownloadFromSelected(Result<Vec<PathBuf>, RpcError>),
    ToggleSelectMode,
    GoToPath(PathBuf),
    TickNextDownload {
        result: Result<(), RpcError>,
        index: usize,
    },
}

impl From<ClientMessage> for Message {
    fn from(value: ClientMessage) -> Self {
        Message::Client(value)
    }
}

impl ClientState {
    pub fn view<'a>(&'a self) -> Element<'a, Message> {
        let tools = self.tools_bar();
        let units = self.units();
        let all = iced::widget::column![tools, units]
            .spacing(10.)
            .width(Length::Fill);
        Container::new(all)
            .padding(10.)
            .center_x(Length::Fill)
            .into()
    }

    fn units(&self) -> scrollable::Scrollable<'_, Message> {
        let units = self
            .units
            .iter()
            .fold(Row::new().spacing(10.), |acc, x| {
                acc.push(x.button(&self.select))
            })
            .wrap();
        let units = Container::new(units)
            .style(|theme| {
                let Palette { primary, .. } = theme.palette();
                container::Style {
                    border: Border {
                        width: 1.,
                        radius: Radius::new(20),
                        color: primary,
                    },
                    ..Default::default()
                }
            })
            .padding(10.);
        scrollable(units).width(Length::Fill)
    }

    fn tools_bar(&self) -> Container<'_, Message> {
        let home = self.home_button();
        let back = self.back_button();
        let selector = self.select_button();
        let download = self.download_button();
        Container::new(row![selector, back, home, download].spacing(5.).wrap())
            .style(|theme| {
                let Palette { primary, .. } = theme.palette();
                container::Style {
                    border: Border {
                        width: 1.,
                        radius: Radius::new(20),
                        color: primary,
                    },
                    ..Default::default()
                }
            })
            .center_x(Length::Fill)
            .padding(12.)
    }

    fn download_button(&self) -> Button<'_, Message> {
        svg_button(IconName::Download.get())
            .on_press(ClientMessage::QueueDownloadFromSelectedStart.into())
    }

    fn select_button(&self) -> Button<'_, Message> {
        svg_button(if self.select.on {
            IconName::Close.get()
        } else {
            IconName::Select.get()
        })
        .on_press(ClientMessage::ToggleSelectMode.into())
    }
    fn back_button(&self) -> Button<'_, Message> {
        let mut path = self.target.clone();
        let msg = path.pop().then_some(ClientMessage::GoToPath(path).into());
        Button::new("back").on_press_maybe(msg)
    }
    fn home_button(&self) -> Button<'_, Message> {
        if self.target == PathBuf::new() {
            go_home_button()
        } else {
            svg_button(IconName::Home.get())
                .on_press(ClientMessage::GoToPath(PathBuf::new()).into())
        }
    }
}

trait UnitViews {
    fn button<'a>(&'a self, selected: &'a Selected) -> MouseArea<'a, Message>;
}

impl UnitViews for Unit {
    fn button<'a>(&'a self, selected: &'a Selected) -> MouseArea<'a, Message> {
        let svg = svg_from_icon_data(self.icon());
        let text = Text::new(self.name());
        let row = row![svg, text].spacing(4.);
        mouse_area(Button::new(row).style(|theme, _| {
            let selected = selected.is_selected(self);
            let Palette {
                background,
                primary,
                text,
                ..
            } = theme.palette();
            Style {
                border: Border {
                    color: if selected { primary } else { background },
                    width: 1.,
                    radius: Radius::new(5.),
                },
                text_color: text,
                ..Default::default()
            }
        }))
        .interaction(Interaction::Pointer)
        .on_release(ClientMessage::UnitClick(self.clone()).into())
        .on_double_click(ClientMessage::UnitDoubleClick(self.clone()).into())
    }
}

fn svg_button<'a>(icon: &'a [u8]) -> Button<'a, Message> {
    Button::new(svg_from_icon_data(icon))
        .style(|_, _| Style {
            border: Border {
                width: 1.,
                radius: Radius::new(2.),
                ..Default::default()
            },
            ..Default::default()
        })
        .padding(7.)
}

impl State {
    pub fn handle_client_msg(&mut self, msg: ClientMessage) -> Task<Message> {
        let state = &mut self.client;
        match msg {
            ClientMessage::PrepareGrpc(rpc_client) => match rpc_client {
                Ok(grpc) => {
                    *state = ClientState::new(grpc.clone());
                    self.page = Page::Client;
                    self.home.show_form = false;
                    Task::future(grpc.ls(PathBuf::new()))
                        .map(|x| ClientMessage::RefreshUnits(x).into())
                }
                Err(err) => {
                    dbg!(err);
                    Task::none()
                }
            },
            ClientMessage::RefreshUnits(units) => {
                match units {
                    Ok(units) => {
                        state.units = units;
                    }
                    Err(err) => {
                        dbg!(err);
                    }
                }
                Task::none()
            }
            ClientMessage::UnitClick(unit) => {
                if state.select.on {
                    state.select.toggle_unit_selection(&unit);
                } else {
                    state.select.toggle_unit_alone_selection(&unit);
                }
                Task::none()
            }
            ClientMessage::UnitDoubleClick(unit) => match (unit.kind, &state.grpc) {
                (UnitKind::Folder, Some(grpc)) => {
                    state.target = unit.path.clone();
                    Task::perform(grpc.clone().ls(unit.path.clone()), move |xs| {
                        ClientMessage::RefreshUnits(xs).into()
                    })
                }
                _ => {
                    println!("opening file {unit:#?} is not supported yet");
                    Task::none()
                }
            },
            ClientMessage::ToggleSelectMode => {
                if state.select.on {
                    state.select.clear();
                } else {
                    state.select.on = true;
                }
                Task::none()
            }
            ClientMessage::GoToPath(path) => {
                state.target = path.clone();
                match &state.grpc {
                    Some(grpc) => Task::perform(grpc.clone().ls(path), |xs| {
                        ClientMessage::RefreshUnits(xs).into()
                    }),
                    None => Task::none(),
                }
            }
            ClientMessage::QueueDownloadFromSelectedStart => {
                let units = state.select.units.clone();
                let Some(grpc) = &state.grpc else {
                    return Task::none();
                };
                state.select.clear();
                let fut = get_download_paths(grpc.clone(), units);
                Task::perform(fut, |x| ClientMessage::QueueDownloadFromSelected(x).into())
            }
            ClientMessage::QueueDownloadFromSelected(paths) => {
                let paths = match paths {
                    Ok(paths) => paths,
                    Err(err) => {
                        dbg!(err);
                        return Task::none();
                    }
                };
                state.downloads.waiting.extend(paths);
                dbg!(&state.downloads);

                let Some(grpc) = state.grpc.clone() else {
                    return Task::none();
                };

                state.downloads.tick_available(grpc)
            }
            ClientMessage::TickNextDownload { result, index } => {
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
                dbg!(&state.downloads);
                state.downloads.tick_available(grpc)
            }
        }
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
