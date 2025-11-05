use iced::{
    Task, Theme,
    widget::{Svg, svg},
};

use crate::{
    client::ClientState,
    home::HomeState,
    server::{ServerMessage, ServerState},
};

use client::ClientMessage;
use home::HomeMessage;
use iced::Element;

pub mod client;
pub mod home;
pub mod server;

#[derive(Default, Clone)]
pub enum Page {
    #[default]
    Home,
    Client,
    Server,
}

fn main() {
    iced::application(State::new, State::update, State::view)
        .title(State::title)
        .theme(State::theme)
        .run()
        .unwrap();
}

#[derive(Default)]
struct State {
    page: Page,
    pub home: HomeState,
    pub server: ServerState,
    pub client: ClientState,
}

#[derive(Clone)]
pub enum Message {
    GoToPage(Page),
    Home(HomeMessage),
    Client(ClientMessage),
    Server(ServerMessage),
}

impl State {
    fn title(&self) -> String {
        String::from("ours")
    }
    fn theme(&self) -> Theme {
        Theme::Dracula
    }
    fn new() -> Self {
        State {
            page: Default::default(),
            ..Default::default()
        }
    }
    fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::GoToPage(page) => {
                self.page = page;
                Task::none()
            }
            Message::Home(msg) => self.handle_home_msg(msg),
            Message::Client(msg) => self.handle_client_msg(msg),
            Message::Server(msg) => self.handle_server_msg(msg),
        }
    }

    pub fn view<'a>(&'a self) -> Element<'a, Message> {
        match self.page {
            Page::Home => self.home.view(),
            Page::Server => self.server.view(),
            Page::Client => self.client.view(),
        }
    }
}

pub fn svg_from_icon_data(icon: &[u8]) -> Svg<'_> {
    let handle = svg::Handle::from_memory(icon.to_vec());
    Svg::new(handle).width(30.)
}
