use std::net::IpAddr;

use crate::Message;
use iced::{
    Alignment, Background, Border, Color, Element, Task,
    border::Radius,
    widget::{
        Button, Container, Text, center, column, container, mouse_area, opaque, row, stack,
        text_input::{self, Style},
    },
};

#[derive(Debug, Clone, Default)]
pub struct ClientPrequistesState {
    ip: String,
    pub valid_ip: Option<IpAddr>,
    pub port: u16,
}

#[derive(Debug, Clone)]
pub enum ClientPrequistesMessage {
    NewIp {
        value: String,
        valid: Option<IpAddr>,
    },
    NewPort(u16),
}

impl ClientPrequistesMessage {
    pub fn handle(self, state: &mut ClientPrequistesState) -> Task<Message> {
        match self {
            ClientPrequistesMessage::NewIp { value, valid } => {
                state.ip = value;
                state.valid_ip = valid;
            }
            ClientPrequistesMessage::NewPort(port) => {
                state.port = port;
            }
        };
        Task::none()
    }
}

impl ClientPrequistesState {
    pub fn view(&self) -> Container<'_, Message> {
        let title = Text::new("choose client address").size(50.).center();
        let ip_input = self.ip_input();
        let port_input = self.port_input();
        let url_input = row![ip_input, port_input].spacing(10.);

        let submit = Text::new("submit")
            .size(60.)
            .center()
            .color(Color::from_rgb(0.0, 1.0, 0.1));
        let submit =
            Button::new(submit).on_press_maybe(if self.valid_ip.is_some_and(|_| self.port != 0) {
                Some(Message::SubmitClientPrequsits)
            } else {
                None
            });
        let cancel = Text::new("cancel")
            .color(Color::from_rgb(1., 0., 0.))
            .size(60.)
            .center();
        let cancel = Button::new(cancel).on_press(Message::ToggleClientModal);
        let buttons = row![submit, cancel].spacing(10.);

        let content = column![title, url_input, buttons]
            .align_x(Alignment::Center)
            .spacing(20.)
            .padding(20.);
        Container::new(content)
            .style(|_| container::Style {
                text_color: Some(Color::WHITE),
                border: Border {
                    color: Color::WHITE,
                    width: 12.,
                    radius: Radius::new(22.),
                },
                background: Some(Background::Color(Color::BLACK)),
                ..Default::default()
            })
            .padding(20.)
    }

    fn port_input(&self) -> text_input::TextInput<'_, Message> {
        text_input::TextInput::new("insert port", &self.port.to_string())
            .size(30.)
            .padding(10.)
            .align_x(Alignment::Center)
            .style(|_, _| STYLE_INPUT)
            .on_input(|x| {
                Message::ClientPrequistes(ClientPrequistesMessage::NewPort(
                    x.parse::<u16>().unwrap_or_default(),
                ))
            })
    }

    fn ip_input(&self) -> text_input::TextInput<'_, Message> {
        text_input::TextInput::new("insert ip", &self.ip.to_string())
            .size(30.)
            .padding(10.)
            .align_x(Alignment::Center)
            .style(|_, _| {
                if self.valid_ip.is_some() {
                    STYLE_INPUT
                } else {
                    Style {
                        value: Color::from_rgb(1.0, 0.1, 0.1),
                        ..STYLE_INPUT
                    }
                }
            })
            .on_input(|x| {
                Message::ClientPrequistes(ClientPrequistesMessage::NewIp {
                    valid: x.parse::<IpAddr>().ok(),
                    value: x,
                })
            })
    }
}

const STYLE_INPUT: Style = Style {
    background: iced::Background::Color(Color::BLACK),
    border: Border {
        color: Color::WHITE,
        width: 2.,
        radius: Radius {
            top_left: 8.,
            top_right: 8.,
            bottom_right: 8.,
            bottom_left: 8.,
        },
    },
    placeholder: Color::from_rgb(0.7, 0.7, 0.7),
    value: Color::from_rgb(0.1, 1., 0.1),
    selection: Color::from_rgb(0.1, 0.1, 0.8),
    icon: Color::from_rgb(0.1, 0.1, 1.),
};

pub fn modal<'a, Message>(
    base: impl Into<Element<'a, Message>>,
    content: impl Into<Element<'a, Message>>,
    on_blur: Message,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    stack![
        base.into(),
        opaque(
            mouse_area(center(opaque(content)).style(|_theme| {
                container::Style {
                    background: Some(Background::Color(Color {
                        a: 0.8,
                        ..Color::BLACK
                    })),
                    ..container::Style::default()
                }
            }))
            .on_press(on_blur)
        )
    ]
    .into()
}
