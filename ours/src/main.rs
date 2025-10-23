use std::{env::args, path::PathBuf};

use client::{ClientMessage, ClientState};
use common::Origin;
use delivery::Delivery;
use get_port::Ops;
use grpc::client::Unit;
use iced::{
    Color, Element, Subscription, Task, exit,
    theme::Style,
    widget::{Container, Text},
    window::{self, Settings},
};
use local_ip_address::local_ip;
use serve::{ServeMessage, ServeState};
use tokio::runtime::Runtime;

use crate::{
    client::download::DownloadMessage, client_prequistes::ClientPrequistesMessage, home::HomeState,
    serve::serve,
};

mod client;
mod client_prequistes;
mod home;
mod serve;

async fn origin() -> Result<Origin, String> {
    let ip = match local_ip() {
        Ok(ip) => ip,
        Err(err) => {
            return Err(err.to_string());
        }
    };
    let Some(port) = get_port::tcp::TcpPort::any(&ip.to_string()) else {
        return Err("could not find availble port".to_string());
    };
    Ok(Origin { ip, port })
}

pub fn main() {
    let mut args = args();
    args.next();

    let rt = Runtime::new().unwrap();
    if rt.block_on(not_server(args)) {
        iced::daemon(State::new, State::update, State::view)
            .subscription(State::subscription)
            .title(State::title)
            .style(|_, _| Style {
                background_color: Color::BLACK,
                text_color: Color::WHITE,
            })
            .run()
            .unwrap();
    }
}

async fn not_server(args: std::env::Args) -> bool {
    let args = match &args.collect::<Vec<_>>()[..] {
        [target, port] => {
            let target = target.parse::<PathBuf>().expect("target should be a path");
            let port = port.parse::<u16>().expect("port should be a u16 number");
            let Origin { ip, .. } = origin().await.unwrap();
            Some((target, ip, port))
        }
        [target] => {
            let target = target.parse::<PathBuf>().expect("target should be a path");
            let Origin { ip, port } = origin().await.unwrap();
            Some((target, ip, port))
        }
        _ => None,
    };
    match args {
        Some((target, ip, port)) => {
            println!("serving {target:#?} at {ip}:{port}");
            serve(target, port).await;
            false
        }
        None => true,
    }
}

struct State {
    serve: Result<ServeState, String>,
    client: ClientState,
    home: HomeState,
    page: Page,
    main_window_id: Option<window::Id>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            main_window_id: None,
            page: Page::Home,
            serve: Err("Not initialized yet".to_string()),
            home: HomeState::default(),
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
    ClientPrequistes(ClientPrequistesMessage),
    Download(DownloadMessage),
    ToggleClientModal,
    SubmitClientPrequsits,
    ToServe,
    ToClient(Vec<Unit>),
    ToHome,
    ErrorHappned(String),
    MainWindowOpened(window::Id),
    WindowClosed(window::Id),
    None,
    SetServeOrigin(Result<Origin, String>),
}

pub async fn error_message(message: String) -> rfd::MessageDialogResult {
    println!("Error : {}", message);
    rfd::AsyncMessageDialog::new()
        .set_level(rfd::MessageLevel::Error)
        .set_title("ours error")
        .set_description(message)
        .show()
        .await
}

impl State {
    fn new() -> (Self, Task<Message>) {
        let (_, task) = window::open(Settings::default());
        (State::default(), task.map(Message::MainWindowOpened))
    }

    fn title(&self, _: window::Id) -> String {
        "ours".to_string()
    }
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::MainWindowOpened(id) => {
                self.main_window_id = Some(id);
                Task::none()
            }
            Message::Serve(serve_message) => serve_message.handle(&mut self.serve),
            Message::Client(client_message) => client_message.handle(&mut self.client),
            Message::ClientPrequistes(message) => message.handle(&mut self.home.client_prequistes),
            Message::ToServe => {
                self.page = Page::Serve;
                Task::future(origin()).map(Message::SetServeOrigin)
            }
            Message::SetServeOrigin(origin) => {
                self.serve = origin.map(ServeState::new);
                Task::none()
            }
            Message::Download(download_message) => download_message.handle(&mut self.client),
            Message::ToClient(units) => {
                self.client.units = units;
                self.page = Page::Client;
                Task::none()
            }
            Message::ToHome => {
                self.page = Page::Home;
                Task::none()
            }
            Message::ToggleClientModal => {
                self.home.show_client_modal = !self.home.show_client_modal;
                Task::none()
            }
            Message::SubmitClientPrequsits => {
                let ip = self.home.client_prequistes.valid_ip.unwrap();
                let port = self.home.client_prequistes.port;
                let delivery = Delivery::new(Origin { ip, port }.to_string());
                self.client.delivery = delivery.clone();
                self.home.show_client_modal = false;
                Task::perform(delivery.ls(PathBuf::new()), move |units| match units {
                    Ok(units) => Message::ToClient(units),
                    Err(err) => Message::ErrorHappned(err.to_string()),
                })
            }
            Message::ErrorHappned(message) => {
                Task::perform(error_message(message), |_| Message::None)
            }
            Message::None => Task::none(),
            Message::WindowClosed(id) => {
                if self.main_window_id.is_some_and(|x| x == id) {
                    self.main_window_id = None;
                    return exit();
                }
                if self.client.download_window.is_some_and(|x| x == id) {
                    self.client.download_window = None;
                    self.client.downloads.clear();
                }
                Task::none()
            }
        }
    }

    fn view(&self, window_id: window::Id) -> Element<'_, Message> {
        if self.client.download_window.is_some_and(|x| x == window_id) {
            return self.client.downloads.view();
        }
        if self.main_window_id.is_some_and(|x| x == window_id) {
            return match self.page {
                Page::Serve => match &self.serve {
                    Ok(serve) => serve.view().into(),
                    Err(err) => Text::new(err.to_string()).into(),
                },
                Page::Client => self.client.view().into(),
                Page::Home => self.home.view().into(),
            };
        }
        println!("loading...");
        Container::new("void").into()
    }

    fn subscription(&self) -> Subscription<Message> {
        window::close_events().map(Message::WindowClosed)
    }
}
