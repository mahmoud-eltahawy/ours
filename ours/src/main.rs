use std::net::IpAddr;

use client::{ClientMessage, ClientState};
use get_port::Ops;
use iced::daemon::Appearance;
use iced::widget::Container;
use iced::{Color, Task};
use local_ip_address::local_ip;
use mode::{ModeMessage, ModeState};
use serve::{ServeMessage, ServeState};

mod client;
mod mode;
mod serve;

pub fn main() -> iced::Result {
    iced::application("ours", State::update, State::view)
        .style(|_, _| Appearance {
            background_color: Color::BLACK,
            text_color: Color::WHITE,
        })
        .run_with(|| {
            let ip: IpAddr = local_ip().unwrap();
            let port = get_port::tcp::TcpPort::any("127.0.0.1").unwrap();
            (State::new(ip, port), Task::none())
        })
}

enum State {
    Serve(ServeState),
    Client(ClientState),
    Mode(ModeState),
}

#[derive(Debug, Clone)]
enum Message {
    Serve(ServeMessage),
    Client(ClientMessage),
    Mode(ModeMessage),
}

impl State {
    fn new(ip: IpAddr, port: u16) -> Self {
        Self::Serve(ServeState::new(ip, port))
    }
    fn update(&mut self, message: Message) -> Task<Message> {
        match (message, self) {
            (Message::Serve(message), State::Serve(ss)) => message.handle(ss),
            (Message::Client(message), State::Client(cs)) => message.handle(cs),
            (Message::Mode(message), State::Mode(sm)) => message.handle(sm),
            _ => unreachable!(),
        }
    }

    fn view(&self) -> Container<Message> {
        match self {
            State::Serve(s) => s.serve_view(),
            State::Client(s) => todo!(),
            State::Mode(s) => todo!(),
        }
    }
}
