use std::net::{IpAddr, Ipv4Addr};

use common::Unit;
use delivery::Delivery;
use iced::{
    Task,
    widget::{Container, Row, Text},
};

use crate::{Message, serve::Origin};

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
        let mut row = Row::new().spacing(20.);
        for x in self.units.iter() {
            row = row.push(Text::new(x.path.to_str().unwrap().to_string()));
        }
        Container::new(row)
    }
}
