use assets::IconName;
use common::Origin;
use delivery::Delivery;
use grpc::{
    UnitKind,
    client::{Selected, Unit},
};
use iced::{
    Border, Color, Element, Length, Task,
    border::Radius,
    mouse::Interaction,
    widget::{
        Button, Container, MouseArea, Row, Svg, Text, button::Style, column, mouse_area, row,
        scrollable, svg::Handle,
    },
    window,
};
use std::{
    net::{IpAddr, Ipv4Addr},
    path::PathBuf,
};

pub mod download;

use crate::{Message, client::download::DownloadMessage, home::go_home_button};

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
    UnitDoubleClick(Unit),
    UnitClick(Unit),
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
            ClientMessage::UnitDoubleClick(unit) => match unit.kind {
                UnitKind::Folder => {
                    Task::perform(state.delivery.clone().ls(unit.path.clone()), move |xs| {
                        if let Ok(xs) = xs {
                            Message::Client(ClientMessage::CurrentPathChanged {
                                units: xs,
                                current_path: unit.path.clone(),
                            })
                        } else {
                            Message::None
                        }
                    })
                }
                _ => {
                    println!("opening file {unit:#?} is not supported yet");
                    Task::none()
                }
            },
            ClientMessage::UnitClick(unit) => {
                if state.select.on {
                    state.select.toggle_unit_selection(&unit);
                } else {
                    state.select.toggle_unit_alone_selection(&unit);
                }
                Task::none()
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Download {
    id: u64,
    server_path: PathBuf,
    host_path: PathBuf,
}

pub struct ClientState {
    pub delivery: Delivery,
    pub units: Vec<Unit>,
    pub current_path: PathBuf,
    pub select: Selected,
    pub download_window: Option<window::Id>,
    pub downloads: download::Downloads,
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
            downloads: download::Downloads::new(),
            download_window: None,
        }
    }
}

impl ClientState {
    pub fn view(&self) -> impl Into<Element<'_, Message>> {
        let tools = self.tools_bar();
        let units = self.units();
        let all = column![tools, units].spacing(10.).width(Length::Fill);
        Container::new(all).padding(10.).center_x(Length::Fill)
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
            .style(|_| iced::widget::container::Style {
                border: Border {
                    color: Color::WHITE,
                    width: 2.,
                    radius: Radius::new(20),
                },
                ..Default::default()
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
            .style(|_| iced::widget::container::Style {
                border: Border {
                    color: Color::WHITE,
                    width: 2.,
                    radius: Radius::new(20),
                },
                ..Default::default()
            })
            .center_x(Length::Fill)
            .padding(12.)
    }

    fn download_button(&self) -> Button<'_, Message> {
        svg_button(IconName::Download.get()).on_press(match self.download_window {
            Some(id) => Message::Download(DownloadMessage::CloseDownloadWindow(id)),
            None => Message::Download(DownloadMessage::OpenDownloadWindow),
        })
    }

    fn select_button(&self) -> Button<'_, Message> {
        svg_button(if self.select.on {
            IconName::Close.get()
        } else {
            IconName::Select.get()
        })
        .on_press(Message::Client(ClientMessage::ToggleSelectMode))
    }
    fn back_button(&self) -> Button<'_, Message> {
        Button::new("back").on_press(Message::Client(ClientMessage::GoBack))
    }
    fn home_button(&self) -> Button<'_, Message> {
        if self.current_path == PathBuf::new() {
            go_home_button()
        } else {
            svg_button(IconName::Home.get()).on_press(Message::Client(
                ClientMessage::ChangeCurrentPath(PathBuf::new()),
            ))
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
        mouse_area(Button::new(row).style(|_, _| {
            let selected = selected.is_selected(self);
            Style {
                border: Border {
                    color: if selected { Color::WHITE } else { Color::BLACK },
                    width: 5.,
                    radius: Radius::new(5.),
                },
                text_color: Color::WHITE,
                ..Default::default()
            }
        }))
        .interaction(Interaction::Pointer)
        .on_release(Message::Client(ClientMessage::UnitClick(self.clone())))
        .on_double_click(Message::Client(ClientMessage::UnitDoubleClick(
            self.clone(),
        )))
    }
}

const VIOLET: Color = Color::from_rgb(127. / 255., 34. / 255., 254. / 255.);

pub fn svg_from_icon_data(icon: &[u8]) -> Svg<'_> {
    let handle = Handle::from_memory(icon.to_vec());
    Svg::new(handle).width(30.)
}

fn svg_button<'a>(icon: &'a [u8]) -> Button<'a, Message> {
    Button::new(svg_from_icon_data(icon))
        .style(|_, _| Style {
            background: Some(iced::Background::Color(Color::BLACK)),
            border: Border {
                color: Color::WHITE,
                width: 2.,
                radius: Radius::new(2.),
            },
            ..Default::default()
        })
        .padding(7.)
}
