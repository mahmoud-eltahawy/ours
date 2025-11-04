use std::{net::SocketAddr, path::PathBuf};

use grpc::{UnitKind, client::RpcClient};
use iced::{
    Task, Theme,
    widget::{Svg, svg},
};

use crate::{
    client::ClientState,
    home::HomeState,
    server::{ServerMessage, ServerState, serve, which_target},
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
            Message::Home(home_message) => match home_message {
                HomeMessage::PortNewInput(port) => {
                    match port {
                        Ok(port) => {
                            self.home.url_form.port = port;
                        }
                        Err(err) => {
                            dbg!(err);
                        }
                    }
                    Task::none()
                }
                HomeMessage::IpNewInput {
                    valid_ip,
                    input_value,
                } => {
                    self.home.url_form.ip = input_value;
                    match valid_ip {
                        Ok(valid_ip) => {
                            self.home.url_form.valid_ip = Some(valid_ip);
                        }
                        Err(err) => {
                            dbg!(err);
                        }
                    }
                    Task::none()
                }
                HomeMessage::SubmitInput(ip_addr, port) => {
                    Task::future(RpcClient::new(SocketAddr::new(ip_addr, port)))
                        .map(|x| ClientMessage::PrepareGrpc(x).into())
                }
                HomeMessage::ToggleInputModal => {
                    self.home.show_form = !self.home.show_form;
                    Task::none()
                }
            },
            Message::Client(client_message) => match client_message {
                ClientMessage::PrepareGrpc(rpc_client) => match rpc_client {
                    Ok(grpc) => {
                        self.client = ClientState::new(grpc.clone());
                        self.page = Page::Client;
                        self.home.show_form = false;
                        Task::future(grpc.ls(PathBuf::new()))
                            .map(|x| ClientMessage::RefreshUnits(x).into())
                    }
                    Err(err) => {
                        dbg!(err);
                        Task::none()
                    }
                },
                ClientMessage::RefreshUnits(units) => {
                    match units {
                        Ok(units) => {
                            self.client.units = units;
                        }
                        Err(err) => {
                            dbg!(err);
                        }
                    }
                    Task::none()
                }
                ClientMessage::UnitClick(unit) => {
                    if self.client.select.on {
                        self.client.select.toggle_unit_selection(&unit);
                    } else {
                        self.client.select.toggle_unit_alone_selection(&unit);
                    }
                    Task::none()
                }
                ClientMessage::UnitDoubleClick(unit) => {
                    match (unit.kind, &self.client.grpc) {
                        (UnitKind::Folder, Some(grpc)) => {
                            self.client.target = unit.path.clone();
                            Task::perform(grpc.clone().ls(unit.path.clone()), move |xs| {
                                ClientMessage::RefreshUnits(xs).into()
                            })
                        }
                        (_, Some(grpc)) => {
                            //TODO : double click on files should open preview them not to download them
                            Task::perform(grpc.clone().download_file(unit.path), |x| {
                                println!("{:#?}", x);
                                ClientMessage::QueueDownloadFromSelected.into()
                            })
                        }
                        _ => {
                            println!("opening file {unit:#?} is not supported yet");
                            Task::none()
                        }
                    }
                }
                ClientMessage::ToggleSelectMode => {
                    if self.client.select.on {
                        self.client.select.clear();
                    } else {
                        self.client.select.on = true;
                    }
                    Task::none()
                }
                ClientMessage::GoToPath(path) => {
                    self.client.target = path.clone();
                    match &self.client.grpc {
                        Some(grpc) => Task::perform(grpc.clone().ls(path), |xs| {
                            ClientMessage::RefreshUnits(xs).into()
                        }),
                        None => Task::none(),
                    }
                }
                ClientMessage::QueueDownloadFromSelected => unimplemented!(),
            },
            Message::Server(server_message) => match server_message {
                ServerMessage::Launch => {
                    self.server.working_process = Some(tokio::spawn(serve(
                        self.server.target_path.clone(),
                        self.server.tonic_port,
                        self.server.axum_port,
                    )));
                    Task::none()
                }
                ServerMessage::Stop => {
                    if let Some(x) = &self.server.working_process {
                        x.abort();
                        self.server.working_process = None;
                    }
                    Task::none()
                }
                ServerMessage::PickTarget => {
                    Task::perform(which_target(), |x| ServerMessage::TargetPicked(x).into())
                }
                ServerMessage::TargetPicked(path_buf) => {
                    if let Some(path_buf) = path_buf {
                        self.server.target_path = path_buf;
                    }
                    Task::none()
                }
            },
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
