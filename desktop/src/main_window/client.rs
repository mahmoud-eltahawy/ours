use crate::{Message, State, main_window::home::HomeMessage};
use iced::{Element, widget::Text};

#[derive(Default)]
pub struct ClientState {}

pub enum ClientMessage {
    HomeMessage(HomeMessage),
}

impl State {
    pub fn client<'a>(&'a self) -> Element<'a, Message> {
        Text::new("client page").into()
    }
}
