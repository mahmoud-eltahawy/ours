use assets::HOME_SVG;
use iced::{
    Alignment, Background, Color, Length,
    border::Radius,
    widget::{Button, Container, Text, button::Style, column},
};

use crate::{Message, client::svg_from_icon_data};

pub fn home_view<'a>() -> Container<'a, Message> {
    let title = Text::new("Welcome Home").size(80).center();
    let message = Text::new("choose the mode").size(70).center();
    let serve = Text::new("server mode").center().size(60);
    let style = Style {
        background: Some(Background::Color(Color::BLACK)),
        text_color: Color::WHITE,
        border: iced::Border {
            color: Color::WHITE,
            width: 5.,
            radius: Radius::new(30.),
        },
        ..Default::default()
    };
    let serve = Button::new(serve)
        .padding(30.)
        .style(move |_, _| style)
        .on_press(Message::ToServe);
    let client = Text::new("client mode").center().size(60);
    let client = Button::new(client)
        .padding(30.)
        .style(move |_, _| style)
        .on_press(Message::GetClientPrequsits);

    let content = column![title, message, serve, client]
        .align_x(Alignment::Center)
        .spacing(20.);
    Container::new(content)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
}

pub fn go_home_button<'a>() -> Button<'a, Message> {
    Button::new(svg_from_icon_data(&HOME_SVG))
        .on_press(Message::ToHome)
        .style(move |_, _| Style {
            background: Some(iced::Background::Color(Color::from_rgb(0.9, 0.1, 0.1))),
            ..Default::default()
        })
}
