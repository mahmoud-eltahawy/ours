use iced::{
    Color,
    widget::{Button, Container, Text, button::Style, column},
};

use crate::Message;

pub fn home_view<'a>() -> Container<'a, Message> {
    let title = Text::new("Home");
    let serve = Button::new("to serve").on_press(Message::ToServe);
    let client = Button::new("to client").on_press(Message::GetClientPrequsits);

    let content = column![title, serve, client].spacing(20.);
    Container::new(content)
}

pub fn go_home_button<'a>() -> Button<'a, Message> {
    Button::new("home")
        .on_press(Message::ToHome)
        .style(move |_, _| Style {
            background: Some(iced::Background::Color(Color::from_rgb(1., 0., 0.))),
            ..Default::default()
        })
}
