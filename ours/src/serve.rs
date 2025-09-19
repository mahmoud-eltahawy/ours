use common::Origin;
use iced::widget::qr_code::Data;
use iced::{
    Background, Border, Center, Color, Shadow, Task, Vector,
    border::Radius,
    widget::{Button, Column, Container, Row, button::Style, column, qr_code, row, text},
};
use iced::{Element, Length};
use rfd::AsyncFileDialog;
use std::env::home_dir;
use std::path::PathBuf;
use tokio::task::JoinHandle;

use crate::Message;
use crate::home::go_home_button;

#[derive(Debug, Clone)]
pub enum ServeMessage {
    Launch,
    Stop,
    PickTarget,
    TargetPicked(PathBuf),
}

pub struct ServeState {
    pub origin: Origin,
    pub target_path: PathBuf,
    pub url: Data,
    pub working_process: Option<JoinHandle<()>>,
}

fn qr_data(origin: &Origin) -> Data {
    Data::new(origin.to_string().into_bytes()).unwrap()
}

impl ServeState {
    pub fn new(origin: Origin) -> Self {
        Self {
            target_path: home_dir().unwrap_or_default(),
            url: qr_data(&origin),
            origin,
            working_process: None,
        }
    }
}

pub async fn serve(root: PathBuf, port: u16) {
    server::Server::new(root).port(port).serve().await.unwrap();
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
                state.working_process = Some(tokio::spawn(serve(
                    state.target_path.clone(),
                    state.origin.port,
                )));
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
    pub fn view(&self) -> impl Into<Element<'_, Message>> {
        let home = go_home_button();
        let serve = self.serve_button();
        let tp = self.target_pick();
        let us = self.url_section();
        let col = column![home, serve, tp, us,]
            .spacing(30)
            .padding(20)
            .align_x(Center);
        Container::new(col).center_x(Length::Fill)
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

    fn url_section(&self) -> Column<'_, Message> {
        let my_text = |x: String| text(x).size(60).align_x(Center).center();
        let at = my_text(String::from("at"));
        let url = my_text(self.origin.to_string());
        let qr = qr_code(&self.url).cell_size(13);
        column![at, url, qr]
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
            .on_press_maybe(if working {
                None
            } else {
                Some(Message::Serve(ServeMessage::PickTarget))
            })
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
                let mes = match &self.working_process {
                    Some(_) => ServeMessage::Stop,
                    None => ServeMessage::Launch,
                };
                Message::Serve(mes)
            })
    }
}
