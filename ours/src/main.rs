use std::net::IpAddr;

use client::{ClientMessage, ClientState};
use iced::daemon::Appearance;
use iced::widget::Container;
use iced::{Color, Task};
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
            // let ip: IpAddr = local_ip().unwrap();
            // let port = get_port::tcp::TcpPort::any("127.0.0.1").unwrap();
            (State::Mode(ModeState {}), Task::none())
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
    ToServe,
    ToClient(ClientState),
    ToMode(ModeState),
    None,
}

impl State {
    fn new(ip: IpAddr, port: u16) -> Self {
        Self::Serve(ServeState::default().with(ip, port))
    }
    fn update(&mut self, message: Message) -> Task<Message> {
        match (message, self) {
            (Message::Serve(message), State::Serve(ss)) => message.handle(ss),
            (Message::Client(message), State::Client(cs)) => message.handle(cs),
            (Message::Mode(message), State::Mode(sm)) => message.handle(sm),
            (Message::ToServe, state) => {
                *state = State::Serve(ServeState::default());
                Task::none()
            }
            (Message::ToClient(client), state) => {
                *state = State::Client(client);
                Task::none()
            }
            (Message::ToMode(mode), state) => {
                *state = State::Mode(mode);
                Task::none()
            }
            _ => unreachable!(),
        }
    }

    fn view(&self) -> Container<Message> {
        match self {
            State::Serve(s) => s.view(),
            State::Client(s) => s.view(),
            State::Mode(s) => s.view(),
        }
    }
}
