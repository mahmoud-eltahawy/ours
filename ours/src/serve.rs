use get_port::Ops;
use iced::widget::qr_code::Data;
use iced::{
    Background, Border, Center, Color, Shadow, Task, Vector,
    border::Radius,
    widget::{Button, Column, Container, Row, button::Style, column, qr_code, row, text},
};
use rfd::AsyncFileDialog;
use std::env::home_dir;
use std::net::Ipv4Addr;
use std::{env::args, fs::canonicalize, net::IpAddr, path::PathBuf};
use tokio::task::JoinHandle;

use crate::Message;

#[derive(Debug, Clone)]
pub enum ServeMessage {
    Launch,
    Stop,
    PickTarget,
    TargetPicked(PathBuf),
}

pub struct ServeState {
    pub ip: IpAddr,
    pub port: u16,
    pub target_path: PathBuf,
    pub url: Data,
    pub working_process: Option<JoinHandle<()>>,
}

impl Default for ServeState {
    fn default() -> Self {
        let ip = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));
        let port = get_port::tcp::TcpPort::any(&ip.to_string()).unwrap();
        Self {
            ip,
            port,
            target_path: home_dir().unwrap_or_default(),
            url: Data::new([]).unwrap(),
            working_process: None,
        }
    }
}

impl ServeState {
    pub fn new(ip: IpAddr, port: u16) -> Self {
        Self {
            ip,
            port,
            url: Data::new(format!("http://{ip}:{port}").into_bytes()).unwrap(),
            ..Default::default()
        }
    }

    pub fn url(&self) -> String {
        format!("http://{}:{}", self.ip, self.port)
    }
}

pub async fn serve(root: PathBuf, port: u16) {
    let mut site = args()
        .next()
        .and_then(|x| x.parse::<PathBuf>().ok())
        .and_then(|x| canonicalize(x).ok())
        .unwrap();
    site.pop();
    site.push("site");

    server::Server::new(site, root)
        .port(port)
        .serve()
        .await
        .unwrap();
}

pub async fn which_target() -> Option<PathBuf> {
    AsyncFileDialog::new()
        .pick_folder()
        .await
        .map(|x| x.path().to_path_buf())
}

impl ServeMessage {
    pub fn handle(self, state: &mut ServeState) -> Task<Message> {
        match self {
            ServeMessage::Launch => {
                state.working_process =
                    Some(tokio::spawn(serve(state.target_path.clone(), state.port)));
                Task::none()
            }
            ServeMessage::Stop => {
                if let Some(x) = &state.working_process {
                    x.abort();
                    state.working_process = None;
                }
                Task::none()
            }
            ServeMessage::PickTarget => Task::perform(which_target(), |x| {
                if let Some(x) = x {
                    Message::Serve(ServeMessage::TargetPicked(x))
                } else {
                    Message::None
                }
            }),
            ServeMessage::TargetPicked(path_buf) => {
                state.target_path = path_buf;
                Task::none()
            }
        }
    }
}

impl ServeState {
    pub fn view(&self) -> Container<'_, Message> {
        let serve = self.serve_button();
        let tp = self.target_pick();
        let us = self.url_section();
        let col = column![serve, tp, us,]
            .spacing(30)
            .padding(20)
            .align_x(Center);
        Container::new(col)
    }

    fn is_working(&self) -> bool {
        self.working_process.is_some()
    }

    fn target_pick(&self) -> Row<Message> {
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

    fn url_section(&self) -> Column<Message> {
        let my_text = |x: String| text(x).size(60).align_x(Center).center();
        let at = my_text(String::from("at"));
        let url = my_text(self.url());
        let qr = qr_code(&self.url).cell_size(13);
        column![at, url, qr]
    }

    fn pick_button(&self) -> Button<Message> {
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
            .on_press_maybe(if working {
                None
            } else {
                Some(Message::Serve(ServeMessage::PickTarget))
            })
    }

    fn serve_button(&self) -> Button<Message> {
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
                let mes = match &self.working_process {
                    Some(_) => ServeMessage::Stop,
                    None => ServeMessage::Launch,
                };
                Message::Serve(mes)
            })
    }
}
