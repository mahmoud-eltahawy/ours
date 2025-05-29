use iced::{
    Task,
    widget::{Button, Container, Text, column},
};

use crate::{Message, client::ClientState};

#[derive(Debug, Clone)]
pub enum ModeMessage {}

impl ModeMessage {
    pub fn handle(self, state: &mut ModeState) -> Task<Message> {
        Task::none()
    }
}

#[derive(Debug, Clone)]
pub struct ModeState {}

impl ModeState {
    pub fn view(&self) -> Container<'_, Message> {
        let serve = Button::new("to serve").on_press(Message::ToServe);
        let client = Button::new("to client").on_press(Message::ToClient(ClientState {}));

        let title = Text::new("mode");
        let content = column![title, serve, client];
        Container::new(content)
    }
}
