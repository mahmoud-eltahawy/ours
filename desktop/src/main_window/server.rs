use crate::Message;
use crate::State;
use iced::Element;
use iced::widget::Text;

#[derive(Default)]
pub struct ServerState {}

impl State {
    pub fn server<'a>(&'a self) -> Element<'a, Message> {
        Text::new("server page").into()
    }
}
