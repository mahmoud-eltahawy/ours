use std::net::{IpAddr, Ipv4Addr};

use assets::{FILE_ICON, FOLDER_ICON};
use common::Unit;
use delivery::Delivery;
use iced::{
    Length, Task,
    widget::{Button, Container, Svg, Text, column, row, scrollable, svg::Handle},
};

use crate::{Message, home::go_home_button, serve::Origin};

#[derive(Debug, Clone)]
pub enum ClientMessage {}

impl ClientMessage {
    pub fn handle(self, state: &mut ClientState) -> Task<Message> {
        Task::none()
    }
}

#[derive(Debug, Clone)]
pub struct ClientState {
    pub delivery: Delivery,
    pub units: Vec<Unit>,
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
        }
    }
}

impl ClientState {
    pub fn view(&self) -> Container<'_, Message> {
        let home = go_home_button();
        let col = self
            .units
            .iter()
            .fold(column![home].spacing(10.), |acc, x| acc.push(x.button()));
        Container::new(scrollable(col).width(Length::Fill))
    }
}

trait UnitViews {
    fn button(&self) -> Button<'_, Message>;
}

impl UnitViews for Unit {
    fn button(&self) -> Button<'_, Message> {
        let icon = match self.kind {
            common::UnitKind::Dirctory => FOLDER_ICON,
            _ => FILE_ICON,
        };
        let icon = Svg::new(Handle::from_memory(icon)).width(30.);
        let text = Text::new(self.name());
        let row = row![icon, text].spacing(4.);
        Button::new(row)
    }
}
