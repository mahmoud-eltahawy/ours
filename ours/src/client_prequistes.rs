use std::net::IpAddr;

use crate::{Message, home::go_home_button};
use iced::{
    Border, Task, Theme,
    theme::Palette,
    widget::{
        Button, Container, Text, column, row,
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
        let home = go_home_button();
        let title = Text::new("choose client address");
        let ip_input = self.ip_input();
        let port_input = self.port_input();
        let url_input = row![ip_input, port_input].spacing(5.);

        let submit =
            Button::new("Go").on_press_maybe(if self.valid_ip.is_some_and(|_| self.port != 0) {
                Some(Message::SubmitClientPrequsits)
            } else {
                None
            });

        let content = column![home, title, url_input, submit];
        Container::new(content)
    }

    fn port_input(&self) -> text_input::TextInput<'_, Message> {
        text_input::TextInput::new("insert port", &self.port.to_string()).on_input(|x| {
            Message::ClientPrequistes(ClientPrequistesMessage::NewPort(
                x.parse::<u16>().unwrap_or_default(),
            ))
        })
    }

    fn ip_input(&self) -> text_input::TextInput<'_, Message> {
        text_input::TextInput::new("insert ip", &self.ip.to_string())
            .style(|theme: &Theme, _| {
                let Palette {
                    background,
                    text,
                    primary,
                    success,
                    danger,
                } = theme.palette();
                let style = Style {
                    background: iced::Background::Color(background),
                    border: Border::default(),
                    icon: primary,
                    placeholder: text,
                    value: success,
                    selection: text,
                };
                if self.valid_ip.is_some() {
                    style
                } else {
                    Style {
                        value: danger,
                        ..style
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
