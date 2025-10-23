use crate::{
    Message, State,
    main_window::{client::ClientMessage, home::HomeMessage, server::ServerMessage},
};
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

#[derive(Default)]
pub struct MainWindowState {
    pub home: home::HomeState,
    pub client: Option<client::ClientState>,
    pub server: server::ServerState,
}

#[derive(Clone)]
pub enum MainWindowMessage {
    Home(HomeMessage),
    Client(ClientMessage),
    Server(ServerMessage),
}

impl State {
    pub fn main_window_view<'a>(&'a self) -> Element<'a, Message> {
        match self.main_window_page {
            Page::Home => self.main_window_state.home.view(),
            Page::Server => self.main_window_state.server.view(),
            Page::Client => self.client(),
        }
    }
}
