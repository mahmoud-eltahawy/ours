use std::net::{IpAddr, Ipv4Addr};

use common::Unit;
use delivery::Delivery;
use iced::{
    Length, Task,
    widget::{Button, Container, Text, column, scrollable},
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
        let mut col = column![home].spacing(10.);

        for x in self.units.iter() {
            col = col.push(Button::new(Text::new(x.name())));
        }
        Container::new(scrollable(col).width(Length::Fill))
    }
}
