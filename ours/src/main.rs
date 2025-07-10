use std::path::PathBuf;

use client::{ClientMessage, ClientState};
use common::ls;
use home::home_view;
use iced::daemon::Appearance;
use iced::widget::Container;
use iced::{Color, Task};
use serve::{Origin, ServeMessage, ServeState};

mod client;
mod home;
mod serve;

pub fn main() -> iced::Result {
    iced::application("ours", State::update, State::view)
        .style(|_, _| Appearance {
            background_color: Color::BLACK,
            text_color: Color::WHITE,
        })
        .run_with(|| (State::Mode, Task::none()))
}

enum State {
    Serve(ServeState),
    Client(ClientState),
    Mode,
}

#[derive(Debug, Clone)]
enum Message {
    Serve(ServeMessage),
    Client(ClientMessage),
    GetClientPrequsits,
    ToClient(ClientState),
    ToServe,
    ToHome,
    None,
}

impl State {
    fn update(&mut self, message: Message) -> Task<Message> {
        match (message, self) {
            (Message::Serve(message), State::Serve(ss)) => message.handle(ss),
            (Message::Client(message), State::Client(cs)) => message.handle(cs),
            (Message::ToServe, state) => {
                *state = State::Serve(ServeState::default());
                Task::none()
            }
            (Message::GetClientPrequsits, _) => {
                let origin = Origin::new();
                Task::perform(ls(origin.to_string(), PathBuf::new()), move |x| {
                    let units = x.unwrap_or_default();
                    let cs = ClientState {
                        origin: origin.clone(),
                        units,
                    };
                    Message::ToClient(cs)
                })
            }
            (Message::ToHome, state) => {
                *state = State::Mode;
                Task::none()
            }
            (Message::ToClient(cs), state) => {
                *state = State::Client(cs);
                Task::none()
            }
            _ => unreachable!(),
        }
    }

    fn view(&self) -> Container<Message> {
        match self {
            State::Serve(s) => s.view(),
            State::Client(s) => s.view(),
            State::Mode => home_view(),
        }
    }
}
