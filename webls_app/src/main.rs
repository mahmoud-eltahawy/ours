use std::net::IpAddr;

use get_port::Ops;
use iced::Background;
use iced::widget::button::Style;
use iced::widget::{Column, button, column, text};
use iced::{Center, Color, Task};
use local_ip_address::local_ip;

pub fn main() -> iced::Result {
    iced::application("webls", State::update, State::view).run_with(|| {
        let ip: IpAddr = local_ip().unwrap();
        let port = get_port::tcp::TcpPort::any("127.0.0.1").unwrap();
        (State::new(ip, port), Task::none())
    })
}

struct State {
    ip: IpAddr,
    port: u16,
    working: bool,
}

impl State {
    fn new(ip: IpAddr, port: u16) -> Self {
        Self {
            ip,
            port,
            working: false,
        }
    }
    fn url(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Launch,
    Stop,
}

impl State {
    fn update(&mut self, message: Message) {
        match message {
            Message::Launch => {
                self.working = true;
            }
            Message::Stop => {
                self.working = false;
            }
        }
    }

    fn view(&self) -> Column<Message> {
        let url = text(format!(
            "{} at url {}",
            if self.working { "serving" } else { "launch" },
            self.url()
        ))
        .size(60)
        .align_x(Center)
        .center();
        let launch = button(if self.working { "stop" } else { "Launch" })
            .style(|_, _| {
                let bg = if self.working {
                    Color::from_rgb(255., 0., 0.)
                } else {
                    Color::from_rgb(0., 255., 0.)
                };
                Style {
                    background: Some(Background::Color(bg)),
                    ..Default::default()
                }
            })
            .on_press(if self.working {
                Message::Stop
            } else {
                Message::Launch
            });
        column![url, launch].padding(20).align_x(Center)
    }
}
