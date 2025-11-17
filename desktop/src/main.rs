use get_port::Ops;
use iced::{
    Task, Theme,
    widget::{Svg, svg},
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

fn main() {
    iced::application(State::new, State::update, State::view)
        .title(State::title)
        .theme(State::theme)
        .run()
        .unwrap();
}

struct State {
    page: Page,
    pub home: home::State,
    pub server: server::State,
    pub client: client::State,
}

#[derive(Clone)]
pub enum Message {
    GoToPage(Page),
    Home(home::Message),
    Client(client::Message),
    Server(server::Message),
}

impl State {
    fn title(&self) -> String {
        String::from("ours")
    }
    fn theme(&self) -> Theme {
        Theme::Dracula
    }
    fn new() -> Self {
        let local_ip = local_ip_address::local_ip().unwrap();
        let tonic_port = get_port::tcp::TcpPort::any(&local_ip.to_string()).unwrap();
        let axum_port =
            get_port::tcp::TcpPort::except(&local_ip.to_string(), vec![tonic_port]).unwrap();

        Self {
            page: Default::default(),
            home: Default::default(),
            server: server::State::new(local_ip, tonic_port, axum_port),
            client: Default::default(),
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
