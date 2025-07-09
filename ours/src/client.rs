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

impl ClientState {
    pub fn view(&self) -> Container<'_, Message> {
        Container::new(Text::new("client"))
    }
}
