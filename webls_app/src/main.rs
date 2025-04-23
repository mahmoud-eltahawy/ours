use std::env::home_dir;
use std::net::IpAddr;
use std::path::PathBuf;
use std::sync::Arc;

use get_port::Ops;
use iced::border::Radius;
use iced::widget::button::Style;
use iced::widget::qr_code;
use iced::widget::qr_code::Data;
use iced::widget::{Button, Column, button, column, text};
use iced::{Background, Border, Shadow, Vector};
use iced::{Center, Color, Task};
use local_ip_address::local_ip;
use rfd::AsyncFileDialog;
use tokio::spawn;
use tokio::task::JoinHandle;

pub fn main() -> iced::Result {
    iced::application("webls", State::update, State::view).run_with(|| {
        let ip: IpAddr = local_ip().unwrap();
        let port = get_port::tcp::TcpPort::any("127.0.0.1").unwrap();
        (State::new(ip, port), Task::none())
    })
}

struct State {
    ip: IpAddr,
    port: u16,
    target_path: Option<PathBuf>,
    working: ServerState,
    url: Data,
}

enum ServerState {
    Working(Arc<JoinHandle<()>>),
    Paused,
}

impl State {
    fn new(ip: IpAddr, port: u16) -> Self {
        let target_path = home_dir();
        Self {
            ip,
            port,
            target_path: target_path.clone(),
            working: ServerState::Paused,
            url: Data::new(format!("{ip}:{port}").into_bytes()).unwrap(),
        }
    }
    fn url(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }
}

#[derive(Debug, Clone)]
enum Message {
    Launch,
    Stop(Arc<JoinHandle<()>>),
    PickTarget,
    TargetPicked(Option<PathBuf>),
}

async fn serve(root: PathBuf, port: u16) {
    let _ = tokio::process::Command::new("./webls")
        .arg(root)
        .arg(port.to_string())
        .output()
        .await
        .unwrap();
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
                let f = serve(self.target_path.clone().unwrap(), self.port);
                let handle: JoinHandle<()> = spawn(f);
                self.working = ServerState::Working(handle.into());
                Task::none()
            }
            Message::Stop(jh) => {
                jh.abort();
                self.working = ServerState::Paused;
                Task::none()
            }
            Message::PickTarget => Task::perform(which_target(), Message::TargetPicked),
            Message::TargetPicked(path_buf) => {
                self.target_path = path_buf;
                Task::none()
            }
        }
    }

    fn view(&self) -> Column<Message> {
        let working = self.is_working();
        let url = text(format!(
            "{} {} at url {}",
            if working { "serving" } else { "serve" },
            self.target_path.clone().unwrap().to_str().unwrap(),
            self.url()
        ))
        .size(60)
        .align_x(Center)
        .center();
        let launch = self.serve_button();
        let pick = button("pick other target").on_press(Message::PickTarget);
        let qr = qr_code(&self.url);
        column![url, launch, pick, qr].padding(20).align_x(Center)
    }

    fn is_working(&self) -> bool {
        !matches!(self.working, ServerState::Paused)
    }

    fn serve_button(&self) -> Button<Message> {
        let working = self.is_working();
        let h = 80.;
        let launch_text = if working { "stop" } else { "serve" };
        let launch_text = text(launch_text)
            .align_x(Center)
            .align_y(Center)
            .size(25.)
            .color(Color::WHITE);
        let launch = button(launch_text)
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
                        color: Color::from_rgb(0.5, 0.7, 0.),
                        offset: Vector::new(0., 0.),
                        blur_radius: h,
                    },
                    ..Default::default()
                }
            })
            .on_press(match &self.working {
                ServerState::Working(join_handle) => Message::Stop(join_handle.clone()),
                ServerState::Paused => Message::Launch,
            });
        launch
    }
}
