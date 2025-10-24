use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use std::{env::home_dir, net::Ipv4Addr};

use crate::Message;
use crate::main_window::MainWindowMessage;
use crate::main_window::home::go_home_button;
use iced::{
    Alignment::Center,
    Background, Border, Color, Element, Length, Shadow, Vector,
    border::Radius,
    widget::{self, Button, Column, Container, Row, button::Style, qr_code, row, text},
};
use rfd::AsyncFileDialog;
use tokio::task::JoinHandle;

pub struct ServerState {
    pub addr: SocketAddr,
    pub target_path: PathBuf,
    pub url: qr_code::Data,
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
        Message::MainWindow(MainWindowMessage::Server(value))
    }
}

impl Default for ServerState {
    fn default() -> Self {
        Self {
            addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8080),
            target_path: home_dir().unwrap(),
            url: qr_code::Data::new("invalid data").unwrap(),
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
        let lt = text(lt).align_x(Center).align_y(Center).size(25.);
        Button::new(lt)
            .height(h)
            .width(h * 1.6)
            .style(move |_, _| {
                let bg = if working {
                    Color::from_rgb(1., 0., 0.)
                } else {
                    Color::from_rgb(0., 1., 0.)
                };
                Style {
                    background: Some(Background::Color(bg)),
                    border: Border {
                        width: 3.,
                        radius: Radius::new(h),
                        color: Color::from_rgb(0., 0., 0.),
                    },
                    shadow: Shadow {
                        color: if working {
                            Color::from_rgb(0.8, 0.5, 0.5)
                        } else {
                            Color::from_rgb(0.5, 0.8, 0.5)
                        },
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
        let my_text = |x: String| text(x).size(60).align_x(Center).center();
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
        let pt = text("pick other target")
            .align_x(Center)
            .align_y(Center)
            .size(25.);

        Button::new(pt)
            .style(move |_, _| {
                let bg = if working {
                    Color::from_rgb(0.1, 0.1, 0.1)
                } else {
                    Color::from_rgb(0.1, 0.1, 1.0)
                };
                let h = 70.;
                Style {
                    background: Some(Background::Color(bg)),
                    border: Border {
                        width: 3.,
                        radius: Radius::new(h),
                        color: Color::from_rgb(0., 0., 0.),
                    },
                    shadow: Shadow {
                        color: Color::from_rgb(0.5, 0.5, 0.8),
                        offset: Vector::new(0., 0.),
                        blur_radius: h * 3.,
                    },
                    ..Default::default()
                }
            })
            .on_press_maybe((!working).then_some(ServerMessage::PickTarget.into()))
    }

    fn url_section(&self) -> Column<'_, Message> {
        let my_text = |x: String| text(x).size(60).align_x(Center).center();
        let at = my_text(String::from("at"));
        let url = my_text(self.addr.to_string());
        let qr = qr_code(&self.url).cell_size(13);
        widget::column![at, url, qr]
    }
}

pub async fn serve(root: PathBuf, port: u16) {
    let one = server::Server::new(root.clone()).port(port - 1).serve();
    let two = grpc::server::RpcServer::new(root, port);
    let two = two.serve();
    let (_, _) = tokio::join!(one, two);
}

pub async fn which_target() -> Option<PathBuf> {
    AsyncFileDialog::new()
        .pick_folder()
        .await
        .map(|x| x.path().to_path_buf())
}
