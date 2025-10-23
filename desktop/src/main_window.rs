use crate::{
    Message, State,
    main_window::{client::ClientMessage, home::HomeMessage},
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
    server: server::ServerState,
}

#[derive(Clone)]
pub enum MainWindowMessage {
    Home(HomeMessage),
    Client(ClientMessage),
}

impl State {
    pub fn main_window_view<'a>(&'a self) -> Element<'a, Message> {
        match self.main_window_page {
            Page::Home => self.main_window_state.home.view(),
            Page::Client => self.client(),
            Page::Server => self.server(),
        }
    }
}
