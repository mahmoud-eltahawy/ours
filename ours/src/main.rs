use std::{env::args, path::PathBuf};

use client::{ClientMessage, ClientState};
use common::Unit;
use delivery::Delivery;
use home::home_view;
use iced::{Color, Task, daemon::Appearance, widget::Container};
use serve::{Origin, ServeMessage, ServeState};

use crate::{
    client_prequistes::{ClientPrequistesMessage, ClientPrequistesState},
    serve::serve,
};

mod client;
mod client_prequistes;
mod home;
mod serve;

#[tokio::main]
pub async fn main() {
    let mut args = args();
    args.next();
    match &args.collect::<Vec<_>>()[..] {
        [target, port] => {
            let target = target.parse::<PathBuf>().expect("target should be a path");
            let port = port.parse::<u16>().expect("port should be a u16 number");
            let Origin { ip, .. } = Origin::random();
            println!("serving at {ip}:{port}");
            serve(target, port).await;
        }
        [target] => {
            let target = target.parse::<PathBuf>().expect("target should be a path");
            let Origin { ip, port } = Origin::random();
            println!("serving at {ip}:{port}");
            serve(target, port).await;
        }
        _ => {
            iced::application("ours", State::update, State::view)
                .style(|_, _| Appearance {
                    background_color: Color::BLACK,
                    text_color: Color::WHITE,
                })
                .run_with(|| (State::default(), Task::none()))
                .unwrap();
        }
    };
}

struct State {
    serve: ServeState,
    client: ClientState,
    client_prequistes: ClientPrequistesState,
    page: Page,
}

impl Default for State {
    fn default() -> Self {
        Self {
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
    GetClientPrequsits,
    SubmitClientPrequsits,
    ToServe,
    ToClient(Vec<Unit>),
    ToHome,
    None,
}

impl State {
    fn update(&mut self, message: Message) -> Task<Message> {
        match (message, &mut self.page) {
            (Message::Serve(message), Page::Serve) => message.handle(&mut self.serve),
            (Message::Client(message), Page::Client) => message.handle(&mut self.client),
            (Message::ClientPrequistes(message), Page::ClientPrequistes) => {
                message.handle(&mut self.client_prequistes)
            }
            (Message::ToServe, page) => {
                *page = Page::Serve;
                self.serve = ServeState::default();
                Task::none()
            }
            (Message::GetClientPrequsits, _) => {
                self.page = Page::ClientPrequistes;
                Task::none()
            }
            (Message::ToHome, page) => {
                *page = Page::Home;
                Task::none()
            }
            (Message::SubmitClientPrequsits, _) => {
                if let Some(ip) = self.client_prequistes.valid_ip {
                    let origin = Origin::new(ip, self.client_prequistes.port);
                    let delivery = Delivery::new(origin.to_string());
                    self.client.delivery = delivery.clone();
                    Task::perform(delivery.ls(PathBuf::new()), move |units| {
                        let units = units.unwrap_or_default();
                        Message::ToClient(units)
                    })
                } else {
                    Task::none()
                }
            }
            (Message::ToClient(units), page) => {
                self.client.units = units;
                *page = Page::Client;
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
            Page::ClientPrequistes => self.client_prequistes.view(),
            Page::Home => home_view(),
        }
    }
}
