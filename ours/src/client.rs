use iced::Task;

use crate::Message;

#[derive(Debug, Clone)]
pub enum ClientMessage {}

impl ClientMessage {
    pub fn handle(self, state: &mut ClientState) -> Task<Message> {
        Task::none()
    }
}

pub struct ClientState {}
