use std::net::IpAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use get_port::Ops;
use iced::Background;
use iced::widget::button::Style;
use iced::widget::{Column, button, column, text};
use iced::{Center, Color, Task};
use local_ip_address::local_ip;
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
    working: ServerState,
}

enum ServerState {
    Working(Arc<JoinHandle<()>>),
    Paused,
}

impl State {
    fn new(ip: IpAddr, port: u16) -> Self {
        Self {
            ip,
            port,
            working: ServerState::Paused,
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
}

async fn serve(_root: PathBuf, _port: u16) {
    println!("starting server");
    tokio::time::sleep(Duration::from_secs(60 * 60)).await;
}

impl State {
    fn update(&mut self, message: Message) {
        match message {
            Message::Launch => {
                let f = serve(PathBuf::from("/home/eltahawy/magit"), self.port);
                let handle: JoinHandle<()> = spawn(f);
                self.working = ServerState::Working(handle.into());
            }
            Message::Stop(jh) => {
                jh.abort();
                self.working = ServerState::Paused;
            }
        }
    }

    fn view(&self) -> Column<Message> {
        let working = !matches!(self.working, ServerState::Paused);
        let url = text(format!(
            "{} at url {}",
            if working { "serving" } else { "launch" },
            self.url()
        ))
        .size(60)
        .align_x(Center)
        .center();
        let launch = button(if working { "stop" } else { "Launch" })
            .style(move |_, _| {
                let bg = if working {
                    Color::from_rgb(255., 0., 0.)
                } else {
                    Color::from_rgb(0., 255., 0.)
                };
                Style {
                    background: Some(Background::Color(bg)),
                    ..Default::default()
                }
            })
            .on_press(match &self.working {
                ServerState::Working(join_handle) => Message::Stop(join_handle.clone()),
                ServerState::Paused => Message::Launch,
            });
        column![url, launch].padding(20).align_x(Center)
    }
}
