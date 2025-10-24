use std::{net::SocketAddr, path::PathBuf};

use grpc::{UnitKind, client::RpcClient};
use iced::{
    Color, Element, Subscription, Task, exit,
    theme::Style,
    widget::{Svg, svg},
    window::{self, Settings},
};

use crate::main_window::{
    MainWindowMessage, Page,
    client::{ClientMessage, ClientState},
    home::HomeMessage,
    server::{ServerMessage, serve, which_target},
};

mod download_window;
mod main_window;

fn main() {
    iced::daemon(State::new, State::update, State::view)
        .subscription(State::close_event)
        .title(State::title)
        .style(|_, _| Style {
            background_color: Color::BLACK,
            text_color: Color::WHITE,
        })
        .run()
        .unwrap();
}

struct State {
    main_window_id: window::Id,
    download_window_id: Option<window::Id>,
    main_window_page: main_window::Page,
    pub main_window_state: main_window::MainWindowState,
}

#[derive(Clone)]
enum Message {
    WindowOpened(window::Id),
    WindowClosed(window::Id),
    GoToPage(main_window::Page),
    MainWindow(MainWindowMessage),
}

impl State {
    fn title(&self, _: window::Id) -> String {
        String::from("ours")
    }
    fn new() -> (Self, Task<Message>) {
        let (id, task) = window::open(Settings::default());
        (
            State {
                main_window_id: id,
                download_window_id: None,
                main_window_page: Default::default(),
                main_window_state: Default::default(),
            },
            task.map(Message::WindowOpened),
        )
    }
    fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::WindowOpened(id) => {
                if self.main_window_id != id {
                    self.download_window_id = Some(id);
                }
                Task::none()
            }
            Message::WindowClosed(id) => {
                if self.download_window_id.is_some_and(|x| x == id) {
                    self.download_window_id = None;
                    Task::none()
                } else {
                    exit()
                }
            }
            Message::GoToPage(page) => {
                self.main_window_page = page;
                Task::none()
            }
            Message::MainWindow(main_window_message) => match main_window_message {
                MainWindowMessage::Home(home_message) => match home_message {
                    HomeMessage::PortNewInput(port) => {
                        self.main_window_state.home.url_form.port = port;
                        Task::none()
                    }
                    HomeMessage::IpNewInput {
                        valid_ip,
                        input_value,
                    } => {
                        self.main_window_state.home.url_form.ip = input_value;
                        self.main_window_state.home.url_form.valid_ip = valid_ip;
                        Task::none()
                    }
                    HomeMessage::SubmitInput(ip_addr, port) => {
                        Task::future(RpcClient::new(SocketAddr::new(ip_addr, port)))
                            .map(|x| ClientMessage::PrepareGrpc(x.ok()).into())
                    }
                    HomeMessage::ToggleInputModal => {
                        self.main_window_state.home.show_form =
                            !self.main_window_state.home.show_form;
                        Task::none()
                    }
                },
                MainWindowMessage::Client(client_message) => match client_message {
                    ClientMessage::PrepareGrpc(rpc_client) => match rpc_client {
                        Some(grpc) => {
                            self.main_window_state.client = ClientState::new(grpc.clone());
                            self.main_window_page = Page::Client;
                            Task::future(grpc.ls(PathBuf::new()))
                                .map(|x| ClientMessage::RefreshUnits(x.unwrap_or_default()).into())
                        }
                        None => Task::none(),
                    },
                    ClientMessage::RefreshUnits(units) => {
                        self.main_window_state.client.units = units;
                        Task::none()
                    }
                    ClientMessage::UnitClick(unit) => {
                        if self.main_window_state.client.select.on {
                            self.main_window_state
                                .client
                                .select
                                .toggle_unit_selection(&unit);
                        } else {
                            self.main_window_state
                                .client
                                .select
                                .toggle_unit_alone_selection(&unit);
                        }
                        Task::none()
                    }
                    ClientMessage::UnitDoubleClick(unit) => {
                        match (unit.kind, &self.main_window_state.client.grpc) {
                            (UnitKind::Folder, Some(grpc)) => {
                                self.main_window_state.client.target = unit.path.clone();
                                Task::perform(grpc.clone().ls(unit.path.clone()), move |xs| {
                                    ClientMessage::RefreshUnits(xs.unwrap_or_default()).into()
                                })
                            }
                            _ => {
                                println!("opening file {unit:#?} is not supported yet");
                                Task::none()
                            }
                        }
                    }
                },
                MainWindowMessage::Server(server_message) => match server_message {
                    ServerMessage::Launch => {
                        self.main_window_state.server.working_process = Some(tokio::spawn(serve(
                            self.main_window_state.server.target_path.clone(),
                            self.main_window_state.server.addr.port(),
                        )));
                        Task::none()
                    }
                    ServerMessage::Stop => {
                        if let Some(x) = &self.main_window_state.server.working_process {
                            x.abort();
                            self.main_window_state.server.working_process = None;
                        }
                        Task::none()
                    }
                    ServerMessage::PickTarget => {
                        Task::perform(which_target(), |x| ServerMessage::TargetPicked(x).into())
                    }
                    ServerMessage::TargetPicked(path_buf) => {
                        if let Some(path_buf) = path_buf {
                            self.main_window_state.server.target_path = path_buf;
                        }
                        Task::none()
                    }
                },
            },
        }
    }

    fn view<'a>(&'a self, window_id: window::Id) -> Element<'a, Message> {
        if self.main_window_id == window_id {
            self.main_window_view()
        } else if self.download_window_id.is_some_and(|x| x == window_id) {
            self.download_window_view()
        } else {
            unreachable!()
        }
    }

    fn close_event(&self) -> Subscription<Message> {
        window::close_events().map(Message::WindowClosed)
    }
}

pub fn svg_from_icon_data(icon: &[u8]) -> Svg<'_> {
    let handle = svg::Handle::from_memory(icon.to_vec());
    Svg::new(handle).width(30.)
}
