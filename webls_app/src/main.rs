use std::env::{home_dir, var};
use std::fs::canonicalize;
use std::net::IpAddr;
use std::path::PathBuf;

use get_port::Ops;
use iced::border::Radius;
use iced::daemon::Appearance;
use iced::widget::button::Style;
use iced::widget::qr_code::Data;
use iced::widget::{Button, Column, button, column, row, text};
use iced::widget::{Row, qr_code};
use iced::{Background, Border, Shadow, Vector};
use iced::{Center, Color, Task};
use local_ip_address::local_ip;
use rfd::AsyncFileDialog;
use tokio::process::Child;

pub fn main() -> iced::Result {
    iced::application("webls", State::update, State::view)
        .style(|_, _| Appearance {
            background_color: Color::BLACK,
            text_color: Color::WHITE,
        })
        .run_with(|| {
            let ip: IpAddr = local_ip().unwrap();
            let port = get_port::tcp::TcpPort::any("127.0.0.1").unwrap();
            (State::new(ip, port), Task::none())
        })
}

struct State {
    ip: IpAddr,
    port: u16,
    target_path: Option<PathBuf>,
    url: Data,
    working_process: Option<Child>,
}

impl State {
    fn new(ip: IpAddr, port: u16) -> Self {
        let target_path = home_dir();
        Self {
            ip,
            port,
            target_path: target_path.clone(),
            url: Data::new(format!("http://{ip}:{port}").into_bytes()).unwrap(),
            working_process: None,
        }
    }
    fn url(&self) -> String {
        format!("http://{}:{}", self.ip, self.port)
    }
}

#[derive(Debug, Clone)]
enum Message {
    Launch,
    Stop,
    PickTarget,
    TargetPicked(Option<PathBuf>),
}

fn serve(root: PathBuf, port: u16) -> Child {
    let key = "LEPTOS_SITE_ROOT";
    let lsr = var(key).unwrap_or_else(|_| panic!("expect variable {key} to exist",));
    let lsr = lsr
        .parse::<PathBuf>()
        .unwrap_or_else(|_| panic!("expected varible {key}={lsr} to be a valid path"));
    let lsr = canonicalize(&lsr)
        .unwrap_or_else(|_| panic!("expected varible {key}={lsr:#?} to be a canonicalable path"));

    let mut wlsr = lsr.clone();
    wlsr.pop();
    wlsr.push("webls");

    tokio::process::Command::new(wlsr)
        .kill_on_drop(true)
        .arg(root)
        .arg(port.to_string())
        .env(key, lsr)
        .spawn()
        .unwrap()
}

async fn which_target() -> Option<PathBuf> {
    AsyncFileDialog::new()
        .pick_folder()
        .await
        .map(|x| x.path().to_path_buf())
}

impl State {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Launch => {
                let child = serve(self.target_path.clone().unwrap(), self.port);
                self.working_process = Some(child);
                Task::none()
            }
            Message::Stop => {
                self.working_process = None;
                Task::none()
            }
            Message::PickTarget => Task::perform(which_target(), Message::TargetPicked),
            Message::TargetPicked(path_buf) => {
                if path_buf.is_some() {
                    self.target_path = path_buf;
                }
                Task::none()
            }
        }
    }

    fn view(&self) -> Column<Message> {
        let serve = self.serve_button();
        let tp = self.target_pick();
        let us = self.url_section();
        column![serve, tp, us,]
            .spacing(30)
            .padding(20)
            .align_x(Center)
    }

    fn is_working(&self) -> bool {
        self.working_process.is_some()
    }

    fn target_pick(&self) -> Row<Message> {
        let my_text = |x: String| text(x).size(60).align_x(Center).center();
        let target = my_text(
            self.target_path
                .clone()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string(),
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
        button(pt)
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
                Some(Message::PickTarget)
            })
    }

    fn serve_button(&self) -> Button<Message> {
        let working = self.is_working();
        let h = 80.;
        let lt = if working { "stop" } else { "serve" };
        let lt = text(lt).align_x(Center).align_y(Center).size(25.);
        let launch = button(lt)
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
            .on_press(match &self.working_process {
                Some(_) => Message::Stop,
                None => Message::Launch,
            });
        launch
    }
}
