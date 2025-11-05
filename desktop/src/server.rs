use common::Origin;
use get_port::Ops;
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
    pub local_ip: Result<IpAddr, local_ip_address::Error>,
    pub axum_port: Option<u16>,
    pub tonic_port: Option<u16>,
    pub target_path: PathBuf,
    pub tonic_qr: Option<qr_code::Data>,
    pub axum_qr: Option<qr_code::Data>,
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

impl Default for ServerState {
    fn default() -> Self {
        let local_ip = local_ip_address::local_ip();
        let tonic_port = local_ip
            .as_ref()
            .ok()
            .and_then(|ip| get_port::tcp::TcpPort::any(&ip.to_string()));

        let axum_port = match tonic_port {
            Some(port) => local_ip
                .as_ref()
                .ok()
                .and_then(|ip| get_port::tcp::TcpPort::except(&ip.to_string(), vec![port])),
            None => local_ip
                .as_ref()
                .ok()
                .and_then(|ip| get_port::tcp::TcpPort::any(&ip.to_string())),
        };

        let tonic_url = local_ip
            .as_ref()
            .ok()
            .and_then(|ip| tonic_port.map(|port| (ip, port)))
            .map(|(ip, port)| Origin::new(*ip, port))
            .and_then(|x| qr_code::Data::new(x.to_string()).ok());
        let axum_url = local_ip
            .as_ref()
            .ok()
            .and_then(|ip| axum_port.map(|port| (ip, port)))
            .map(|(ip, port)| Origin::new(*ip, port))
            .and_then(|x| qr_code::Data::new(x.to_string()).ok());
        Self {
            local_ip,
            axum_port,
            tonic_port,
            target_path: home_dir().unwrap(),
            tonic_qr: tonic_url,
            axum_qr: axum_url,
            working_process: Default::default(),
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
            self.target_path
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
            &self.local_ip,
            self.tonic_port,
            "native app : ",
        ));
        let web_url = my_text(address_msg(&self.local_ip, self.axum_port, "web app : "));
        let native_qr = match &self.tonic_qr {
            None => Container::new(text::Text::new("url is unvalid")),
            Some(x) => Container::new(qr_code(x).cell_size(13)),
        };
        let web_qr = match &self.axum_qr {
            None => Container::new(text::Text::new("url is unvalid")),
            Some(x) => Container::new(qr_code(x).cell_size(13)),
        };
        let web = column![native_url, native_qr].spacing(5.);
        let native = column![web_url, web_qr].spacing(5.);
        let row = row![web, native].spacing(15.);
        widget::column![at, row]
    }
}
fn address_msg(
    local_ip: &Result<IpAddr, local_ip_address::Error>,
    port: Option<u16>,
    prefix: &str,
) -> String {
    match (local_ip, port) {
        (Ok(ip), Some(port)) => format!("{prefix} {}", Origin::new(*ip, port)),
        (Ok(_), None) => format!("{prefix} Error : unavilable port"),
        (Err(err), Some(_)) => format!("{prefix} error while getting local ip : {err:#?}"),
        (Err(err), None) => {
            format!("{prefix} error while getting local ip : {err:#?} and no avilable port")
        }
    }
}

pub async fn which_target() -> Option<PathBuf> {
    AsyncFileDialog::new()
        .pick_folder()
        .await
        .map(|x| x.path().to_path_buf())
}

pub async fn serve(target_path: PathBuf, tonic_port: Option<u16>, axum_port: Option<u16>) {
    let (Some(tonic_port), Some(axum_port)) = (tonic_port, axum_port) else {
        return;
    };
    let one = server::Server::new(target_path.clone())
        .port(axum_port)
        .serve();
    let two = grpc::server::RpcServer::new(target_path.clone(), tonic_port);
    let two = two.serve();
    let (_, _) = tokio::join!(one, two);
}

impl State {
    pub fn handle_server_msg(&mut self, msg: ServerMessage) -> Task<Message> {
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
}
