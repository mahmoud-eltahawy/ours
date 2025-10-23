use super::{Message, State};
use iced::{Element, widget::Text};

impl State {
    pub fn download_window_view<'a>(&'a self) -> Element<'a, Message> {
        Text::new("download page").into()
    }
}
