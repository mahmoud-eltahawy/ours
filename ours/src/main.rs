use std::{env::args, path::PathBuf};

use client::{ClientMessage, ClientState};
use common::{Origin, Unit};
use delivery::Delivery;
use home::home_view;
use iced::{
    Color, Subscription, Task, exit,
    theme::Style,
    widget::Container,
    window::{self, Settings},
};
use serve::{ServeMessage, ServeState};
use tokio::runtime::Runtime;

use crate::{
    client::download::DownloadMessage,
    client_prequistes::{ClientPrequistesMessage, ClientPrequistesState},
    serve::serve,
};

mod client;
mod client_prequistes;
mod home;
mod serve;

pub fn main() {
    let mut args = args();
    args.next();

    let args = match &args.collect::<Vec<_>>()[..] {
        [target, port] => {
            let target = target.parse::<PathBuf>().expect("target should be a path");
            let port = port.parse::<u16>().expect("port should be a u16 number");
            let Origin { ip, .. } = Origin::random();
            Some((target, ip, port))
        }
        [target] => {
            let target = target.parse::<PathBuf>().expect("target should be a path");
            let Origin { ip, port } = Origin::random();
            Some((target, ip, port))
        }
        _ => None,
    };

    match args {
        Some((target, ip, port)) => {
            let rt = Runtime::new().unwrap();
            println!("serving {target:#?} at {ip}:{port}");
            rt.block_on(async move {
                serve(target, port).await;
            })
        }
        None => {
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
}

struct State {
    serve: ServeState,
    client: ClientState,
    client_prequistes: ClientPrequistesState,
    page: Page,
    main_window_id: Option<window::Id>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            main_window_id: None,
            page: Page::Home,
            serve: ServeState::default(),
            client: ClientState::default(),
            client_prequistes: ClientPrequistesState::default(),
        }
    }
}

enum Page {
    Serve,
    Client,
    ClientPrequistes,
    Home,
}

#[derive(Debug, Clone)]
enum Message {
    Serve(ServeMessage),
    Client(ClientMessage),
    ClientPrequistes(ClientPrequistesMessage),
    Download(DownloadMessage),
    GetClientPrequsits,
    SubmitClientPrequsits,
    ToServe,
    ToClient(Vec<Unit>),
    ToHome,
    ErrorHappned(String),
    MainWindowOpened(window::Id),
    WindowClosed(window::Id),
    None,
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
            Message::ClientPrequistes(message) => message.handle(&mut self.client_prequistes),
            Message::ToServe => {
                self.page = Page::Serve;
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
            Message::GetClientPrequsits => {
                self.page = Page::ClientPrequistes;
                Task::none()
            }
            Message::SubmitClientPrequsits => {
                if let Some(ip) = self.client_prequistes.valid_ip {
                    let origin = Origin::new(ip, self.client_prequistes.port);
                    let delivery = Delivery::new(origin.to_string());
                    self.client.delivery = delivery.clone();
                    Task::perform(delivery.ls(PathBuf::new()), move |units| match units {
                        Ok(units) => Message::ToClient(units),
                        Err(err) => Message::ErrorHappned(err.to_string()),
                    })
                } else {
                    Task::none()
                }
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

    fn view(&self, window_id: window::Id) -> Container<'_, Message> {
        if self.client.download_window.is_some_and(|x| x == window_id) {
            return self.client.downloads.view();
        }
        if self.main_window_id.is_some_and(|x| x == window_id) {
            return match self.page {
                Page::Serve => self.serve.view(),
                Page::Client => self.client.view(),
                Page::ClientPrequistes => self.client_prequistes.view(),
                Page::Home => home_view(),
            };
        }
        println!("loading...");
        Container::new("void")
    }

    fn subscription(&self) -> Subscription<Message> {
        window::close_events().map(Message::WindowClosed)
    }
}
