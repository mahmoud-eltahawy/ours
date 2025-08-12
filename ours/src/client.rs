use std::net::{IpAddr, Ipv4Addr};

use common::Unit;
use iced::{
    Task,
    widget::{Container, Text},
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
    pub origin: Origin,
    pub units: Vec<Unit>,
}

impl Default for ClientState {
    fn default() -> Self {
        Self {
            origin: Origin {
                ip: IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
                port: 80,
            },
            units: Vec::new(),
        }
    }
}

impl ClientState {
    pub fn view(&self) -> Container<'_, Message> {
        Container::new(Text::new("client"))
    }
}
