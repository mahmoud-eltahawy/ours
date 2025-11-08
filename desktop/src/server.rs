use common::Origin;
use grpc::server::RpcServer;
use iced::Task;
use std::env::home_dir;
use std::net::IpAddr;
use std::path::PathBuf;

use crate::home::go_home_button;
use crate::{Message, State};
use iced::{
    Alignment::Center,
    Background, Border, Element, Length, Shadow, Theme, Vector,
    border::Radius,
    theme::Palette,
    widget::{
        self, Button, Column, Container, Row,
        button::Style,
        column, qr_code, row,
        text::{self, Wrapping},
    },
};
use rfd::AsyncFileDialog;
use tokio::task::JoinHandle;

pub struct ServerState {
    pub web_origin: Origin,
    pub rpc_server: RpcServer,
    pub tonic_qr: qr_code::Data,
    pub axum_qr: qr_code::Data,
    pub working_process: Option<JoinHandle<()>>,
}

#[derive(Debug, Clone)]
pub enum ServerMessage {
    Launch,
    Stop,
    PickTarget,
    TargetPicked(Option<PathBuf>),
}

impl From<ServerMessage> for Message {
    fn from(value: ServerMessage) -> Self {
        Message::Server(value)
    }
}

impl ServerState {
    pub fn new(local_ip: IpAddr, tonic_port: u16, axum_port: u16) -> Self {
        let mut origin = Origin::new(local_ip, tonic_port);
        let tonic_url = qr_code::Data::new(origin.to_string()).unwrap();
        origin.port = axum_port;
        let axum_url = qr_code::Data::new(origin.to_string()).unwrap();
        Self {
            web_origin: origin,
            tonic_qr: tonic_url,
            axum_qr: axum_url,
            working_process: None,
            rpc_server: RpcServer {
                target_dir: home_dir().unwrap(),
                port: tonic_port,
            },
        }
    }
}

impl ServerState {
    pub fn view<'a>(&'a self) -> Element<'a, Message> {
        let home = go_home_button();
        let serve = self.serve_button();
        let tp = self.target_pick();
        let us = self.url_section();

        let col = widget::column![home, serve, tp, us]
            .spacing(30)
            .padding(20)
            .align_x(Center);
        Container::new(col).center_x(Length::Fill).into()
    }

    fn serve_button(&self) -> Button<'_, Message> {
        let working = self.is_working();
        let h = 80.;
        let lt = if working { "stop" } else { "serve" };
        Button::new(
            text::Text::new(lt)
                .align_x(Center)
                .align_y(Center)
                .size(25.),
        )
        .height(h)
        .width(h * 1.6)
        .style(move |theme: &Theme, _| {
            let Palette {
                background,
                primary,
                success,
                warning,
                danger,
                ..
            } = theme.palette();
            let bg = if working { danger } else { success };
            Style {
                background: Some(Background::Color(bg)),
                border: Border {
                    width: 3.,
                    radius: Radius::new(h),
                    color: background,
                },
                shadow: Shadow {
                    color: if working { warning } else { primary },
                    offset: Vector::new(0., 0.),
                    blur_radius: h * 3.,
                },
                ..Default::default()
            }
        })
        .on_press({
            match &self.working_process {
                Some(_) => ServerMessage::Stop,
                None => ServerMessage::Launch,
            }
            .into()
        })
    }

    fn is_working(&self) -> bool {
        self.working_process.is_some()
    }

    fn target_pick(&self) -> Row<'_, Message> {
        let my_text = |x: String| text::Text::new(x).size(60).align_x(Center).center();
        let target = my_text(
            self.rpc_server
                .target_dir
                .clone()
                .to_str()
                .map(|x| x.to_string())
                .unwrap_or_default(),
        );
        let or = my_text(String::from("or"));
        let pick = self.pick_button();
        row![target, or, pick].align_y(Center).spacing(20.)
    }

    fn pick_button(&self) -> Button<'_, Message> {
        let working = self.is_working();
        let pt = text::Text::new("pick other target")
            .align_x(Center)
            .align_y(Center)
            .size(25.);

        Button::new(pt)
            .style(move |theme: &Theme, _| {
                let Palette {
                    background,
                    success,
                    ..
                } = theme.palette();

                let bg = if working { background } else { success };
                let h = 70.;
                Style {
                    background: Some(Background::Color(bg)),
                    border: Border {
                        width: 3.,
                        radius: Radius::new(h),
                        color: bg,
                    },
                    shadow: Shadow {
                        color: success,
                        offset: Vector::new(0., 0.),
                        blur_radius: h * 3.,
                    },
                    ..Default::default()
                }
            })
            .on_press_maybe((!working).then_some(ServerMessage::PickTarget.into()))
    }

    fn url_section(&self) -> Column<'_, Message> {
        let my_text = |x: String| {
            text::Text::new(x)
                .wrapping(Wrapping::Word)
                .size(40)
                .align_x(Center)
                .center()
        };
        let at = my_text(String::from("at"));
        let native_url = my_text(address_msg(
            &self.web_origin.ip,
            self.rpc_server.port,
            "native app : ",
        ));
        let web_url = my_text(address_msg(
            &self.web_origin.ip,
            self.web_origin.port,
            "web app : ",
        ));
        let native_qr = Container::new(qr_code(&self.tonic_qr).cell_size(13));
        let web_qr = Container::new(qr_code(&self.axum_qr).cell_size(13));
        let web = column![native_url, native_qr].spacing(5.);
        let native = column![web_url, web_qr].spacing(5.);
        let row = row![web, native].spacing(15.);
        widget::column![at, row]
    }
}
fn address_msg(local_ip: &IpAddr, port: u16, prefix: &str) -> String {
    format!("{prefix} {}", Origin::new(*local_ip, port))
}

pub async fn which_target() -> Option<PathBuf> {
    AsyncFileDialog::new()
        .pick_folder()
        .await
        .map(|x| x.path().to_path_buf())
}

pub async fn serve(target_path: PathBuf, tonic_port: u16, axum_port: u16) {
    let one = server::Server::new(target_path.clone())
        .port(axum_port)
        .serve();
    let two = RpcServer::new(target_path.clone(), tonic_port);
    let two = two.serve();
    let (_, _) = tokio::join!(one, two);
}

impl State {
    pub fn handle_server_msg(&mut self, msg: ServerMessage) -> Task<Message> {
        let state = &mut self.server;
        match msg {
            ServerMessage::Launch => {
                state.working_process = Some(tokio::spawn(serve(
                    state.rpc_server.target_dir.clone(),
                    state.rpc_server.port,
                    state.web_origin.port,
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
                    state.rpc_server.target_dir = path_buf;
                }
                Task::none()
            }
        }
    }
}
