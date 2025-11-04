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
            Message::Home(msg) => self.handle_home_msg(msg),
            Message::Client(msg) => self.handle_client_msg(msg),
            Message::Server(msg) => self.handle_server_msg(msg),
        }
    }

    fn handle_server_msg(&mut self, msg: ServerMessage) -> Task<Message> {
        let state = &mut self.server;
        match msg {
            ServerMessage::Launch => {
                state.working_process = Some(tokio::spawn(serve(
                    state.target_path.clone(),
                    state.tonic_port,
                    state.axum_port,
                )));
                Task::none()
            }
            ServerMessage::Stop => {
                if let Some(x) = &state.working_process {
                    x.abort();
                    state.working_process = None;
                }
                Task::none()
            }
            ServerMessage::PickTarget => {
                Task::perform(which_target(), |x| ServerMessage::TargetPicked(x).into())
            }
            ServerMessage::TargetPicked(path_buf) => {
                if let Some(path_buf) = path_buf {
                    state.target_path = path_buf;
                }
                Task::none()
            }
        }
    }

    fn handle_client_msg(&mut self, msg: ClientMessage) -> Task<Message> {
        let state = &mut self.client;
        match msg {
            ClientMessage::PrepareGrpc(rpc_client) => match rpc_client {
                Ok(grpc) => {
                    *state = ClientState::new(grpc.clone());
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
                        state.units = units;
                    }
                    Err(err) => {
                        dbg!(err);
                    }
                }
                Task::none()
            }
            ClientMessage::UnitClick(unit) => {
                if state.select.on {
                    state.select.toggle_unit_selection(&unit);
                } else {
                    state.select.toggle_unit_alone_selection(&unit);
                }
                Task::none()
            }
            ClientMessage::UnitDoubleClick(unit) => {
                match (unit.kind, &state.grpc) {
                    (UnitKind::Folder, Some(grpc)) => {
                        state.target = unit.path.clone();
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
                if state.select.on {
                    state.select.clear();
                } else {
                    state.select.on = true;
                }
                Task::none()
            }
            ClientMessage::GoToPath(path) => {
                state.target = path.clone();
                match &state.grpc {
                    Some(grpc) => Task::perform(grpc.clone().ls(path), |xs| {
                        ClientMessage::RefreshUnits(xs).into()
                    }),
                    None => Task::none(),
                }
            }
            ClientMessage::QueueDownloadFromSelected => unimplemented!(),
        }
    }

    fn handle_home_msg(&mut self, msg: HomeMessage) -> Task<Message> {
        let state = &mut self.home;
        match msg {
            HomeMessage::PortNewInput(port) => {
                match port {
                    Ok(port) => {
                        state.url_form.port = port;
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
                state.url_form.ip = input_value;
                match valid_ip {
                    Ok(valid_ip) => {
                        state.url_form.valid_ip = Some(valid_ip);
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
                state.show_form = !state.show_form;
                Task::none()
            }
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
