use assets::{CLOSE_SVG, IconData, SELECT_SVG};
use common::{Origin, Selected, Unit};
use delivery::Delivery;
use iced::{
    Border, Color, Length, Task,
    theme::Palette,
    widget::{
        Button, Column, Container, Svg, Text, button::Style, column, row, scrollable, svg::Handle,
    },
};
use std::{
    env::home_dir,
    net::{IpAddr, Ipv4Addr},
    path::PathBuf,
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
    DownloadSelected,
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
            ClientMessage::DownloadSelected => {
                let units = state.select.units.clone();
                state.select.clear();
                state.downloading = true;
                Task::perform(
                    state
                        .delivery
                        .clone()
                        .download(units, home_dir().map(|x| x.join("Downloads")).unwrap()),
                    move |x| match x {
                        Ok(_) => Message::Client(ClientMessage::DownloadDone),
                        Err(err) => Message::ErrorHappned(err.to_string()),
                    },
                )
            }
            ClientMessage::DownloadDone => {
                state.downloading = false;
                Task::none()
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ClientState {
    pub delivery: Delivery,
    pub units: Vec<Unit>,
    pub current_path: PathBuf,
    pub select: Selected,
    pub downloading: bool,
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
            downloading: false,
        }
    }
}

impl ClientState {
    pub fn view(&self) -> Container<'_, Message> {
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
        let download = Button::new(if self.downloading {
            "downloading"
        } else {
            "download"
        })
        .on_press(Message::Client(ClientMessage::DownloadSelected));
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
