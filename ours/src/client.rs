use assets::{CLOSE_SVG, IconData, SELECT_SVG};
use common::{Origin, Selected, Unit};
use delivery::Delivery;
use iced::{
    Border, Color, Length, Task,
    border::Radius,
    theme::Palette,
    widget::{
        Button, Container, Row, Svg, Text, button::Style, column, row, scrollable, svg::Handle,
    },
    window,
};
use std::{
    net::{IpAddr, Ipv4Addr},
    path::PathBuf,
};

pub mod download;

use crate::{Message, client::download::DownloadMessage};

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
    pub fn view(&self) -> Container<'_, Message> {
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
        Button::new(match self.download_window {
            Some(_) => "close download window",
            None => "open download window",
        })
        .on_press(match self.download_window {
            Some(id) => Message::Download(DownloadMessage::CloseDownloadWindow(id)),
            None => Message::Download(DownloadMessage::OpenDownloadWindow),
        })
    }

    fn select_button(&self) -> Button<'_, Message> {
        Button::new(svg_from_icon_data(if self.select.on {
            &CLOSE_SVG
        } else {
            &SELECT_SVG
        }))
        .on_press(Message::Client(ClientMessage::ToggleSelectMode))
    }
    fn back_button(&self) -> Button<'_, Message> {
        Button::new("back").on_press(Message::Client(ClientMessage::GoBack))
    }
    fn home_button(&self) -> Button<'_, Message> {
        let (message, color) = if self.current_path == PathBuf::new() {
            (Message::ToHome, Color::from_rgb(1., 0., 0.))
        } else {
            (
                Message::Client(ClientMessage::ChangeCurrentPath(PathBuf::new())),
                Color::from_rgb(0., 1., 0.),
            )
        };
        Button::new("home")
            .style(move |_, _| Style {
                background: Some(iced::Background::Color(color)),
                ..Default::default()
            })
            .on_press(message)
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
