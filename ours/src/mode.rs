use iced::Task;

use crate::Message;

#[derive(Debug, Clone)]
pub enum ModeMessage {}

impl ModeMessage {
    pub fn handle(self, state: &mut ModeState) -> Task<Message> {
        Task::none()
    }
}

pub struct ModeState {}
