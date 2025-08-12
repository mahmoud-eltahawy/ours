use std::path::PathBuf;
use std::sync::LazyLock;

use client::{ClientMessage, ClientState};
use delivery::Delivery;
use home::home_view;
use iced::{Color, Task, daemon::Appearance, widget::Container};
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
        .run_with(|| (State::default(), Task::none()))
}

struct State {
    serve: ServeState,
    client: ClientState,
    page: Page,
}

impl Default for State {
    fn default() -> Self {
        Self {
            page: Page::Home,
            serve: ServeState::default(),
            client: ClientState::default(),
        }
    }
}

enum Page {
    Serve,
    Client,
    Home,
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

//FIX : Origin should be retrieved from user input
pub static DELIVERY: LazyLock<Delivery> =
    LazyLock::new(|| Delivery::new(Origin::new().to_string()));

impl State {
    fn update(&mut self, message: Message) -> Task<Message> {
        match (message, &mut self.page) {
            (Message::Serve(message), Page::Serve) => message.handle(&mut self.serve),
            (Message::Client(message), Page::Client) => message.handle(&mut self.client),
            (Message::ToServe, page) => {
                *page = Page::Serve;
                self.serve = ServeState::default();
                Task::none()
            }
            (Message::GetClientPrequsits, _) => {
                let origin = Origin::new();
                Task::perform(DELIVERY.ls(PathBuf::new()), move |x| {
                    let units = x.unwrap_or_default();
                    let cs = ClientState {
                        origin: origin.clone(),
                        units,
                    };
                    Message::ToClient(cs)
                })
            }
            (Message::ToHome, page) => {
                *page = Page::Home;
                Task::none()
            }
            (Message::ToClient(cs), page) => {
                *page = Page::Client;
                self.client = cs;
                Task::none()
            }
            (Message::None, _) => Task::none(),
            _ => unreachable!(),
        }
    }

    fn view(&self) -> Container<'_, Message> {
        match self.page {
            Page::Serve => self.serve.view(),
            Page::Client => self.client.view(),
            Page::Home => home_view(),
        }
    }
}
