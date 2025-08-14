use std::{
    net::{IpAddr, Ipv4Addr},
    path::PathBuf,
};

use common::{Retype, SortUnits, Unit};
use delivery::Delivery;
use iced::{
    Length, Task,
    widget::{Button, Column, Container, Svg, Text, column, row, scrollable, svg::Handle},
};

use crate::{Message, home::go_home_button, serve::Origin};

#[derive(Debug, Clone)]
pub enum ClientMessage {
    ChangeCurrentPath(PathBuf),
    CurrentPathChanged {
        current_path: PathBuf,
        units: Vec<Unit>,
    },
    GoBack,
    GoneBack(Vec<Unit>),
    None,
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
                        Message::Client(ClientMessage::None)
                    }
                })
            }
            ClientMessage::CurrentPathChanged {
                current_path,
                mut units,
            } => {
                units.retype();
                units.sort_units();
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
                            Message::Client(ClientMessage::None)
                        }
                    })
                } else {
                    Task::none()
                }
            }
            ClientMessage::GoneBack(mut units) => {
                units.retype();
                units.sort_units();
                state.current_path.pop();
                state.units = units;
                Task::none()
            }
            ClientMessage::None => Task::none(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ClientState {
    pub delivery: Delivery,
    pub units: Vec<Unit>,
    pub current_path: PathBuf,
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
        }
    }
}

impl ClientState {
    pub fn view(&self) -> Container<'_, Message> {
        let home = go_home_button();
        let back = Button::new("back").on_press(Message::Client(ClientMessage::GoBack));
        let tools = column![back, home].spacing(5.);
        let units = self
            .units
            .iter()
            .fold(Column::new().spacing(10.), |acc, x| acc.push(x.button()));
        let units = scrollable(units).width(Length::Fill);
        let all = column![tools, units].spacing(14.);
        Container::new(scrollable(all).width(Length::Fill))
    }
}

trait UnitViews {
    fn button(&self) -> Button<'_, Message>;
}

impl UnitViews for Unit {
    fn button(&self) -> Button<'_, Message> {
        let handle = Handle::from_memory(self.icon().data.bytes().collect::<Vec<_>>());
        let icon = Svg::new(handle).width(30.);
        let text = Text::new(self.name());
        let row = row![icon, text].spacing(4.);
        Button::new(row).on_press(Message::Client(ClientMessage::ChangeCurrentPath(
            self.path.clone(),
        )))
    }
}
