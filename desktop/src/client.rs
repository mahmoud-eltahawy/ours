use crate::client::downloads::{DownloadMessage, Downloads};
use crate::home::{go_home_button, modal};
use crate::{Message, Page, State, svg_from_icon_data};
use common::assets::IconName;
use grpc::UnitKind;
use grpc::client::RpcClient;
use grpc::error::RpcError;
use grpc::top::{Selected, Unit};
use iced::Alignment;
use iced::Task;
use iced::theme::Palette;
use iced::widget::{Column, container};
use iced::{
    Border, Element, Length,
    border::Radius,
    mouse::Interaction,
    widget::{Button, Container, MouseArea, Row, Text, button::Style, mouse_area, row, scrollable},
};
use std::path::PathBuf;

mod downloads;

#[derive(Default)]
pub struct ClientState {
    pub grpc: Option<RpcClient>,
    pub target: PathBuf,
    pub select: Selected,
    pub units: Vec<Unit>,
    downloads: Downloads,
}

impl ClientState {
    pub fn new(grpc: RpcClient) -> Self {
        Self {
            grpc: Some(grpc),
            target: PathBuf::new(),
            units: Vec::new(),
            select: Selected::default(),
            downloads: Downloads::default(),
        }
    }
}

#[derive(Clone)]
pub enum ClientMessage {
    RefreshUnits(Result<Vec<Unit>, RpcError>),
    PrepareGrpc(Result<RpcClient, RpcError>),
    UnitClick(Unit),
    UnitDoubleClick(Unit),
    ToggleSelectMode,
    GoToPath(PathBuf),
    Download(DownloadMessage),
}

impl From<DownloadMessage> for Message {
    fn from(value: DownloadMessage) -> Self {
        Message::Client(ClientMessage::Download(value))
    }
}

impl From<ClientMessage> for Message {
    fn from(value: ClientMessage) -> Self {
        Message::Client(value)
    }
}

impl ClientState {
    pub fn view<'a>(&'a self) -> Element<'a, Message> {
        let tools = self.tools_bar();
        let units = self.units();
        let all = iced::widget::column![tools, units]
            .spacing(10.)
            .width(Length::Fill);
        let res = Container::new(all)
            .padding(10.)
            .center_x(Length::Fill)
            .into();

        if self.downloads.show_preview {
            modal(
                res,
                self.downloads.view(),
                DownloadMessage::TogglePreview.into(),
            )
        } else {
            res
        }
    }

    fn units(&self) -> scrollable::Scrollable<'_, Message> {
        let units = self
            .units
            .iter()
            .fold(Row::new().spacing(10.), |acc, x| {
                acc.push(x.button(&self.select))
            })
            .wrap();
        let units = Container::new(units)
            .style(|theme| {
                let Palette { primary, .. } = theme.palette();
                container::Style {
                    border: Border {
                        width: 1.,
                        radius: Radius::new(20),
                        color: primary,
                    },
                    ..Default::default()
                }
            })
            .padding(10.);
        scrollable(units).width(Length::Fill)
    }

    fn tools_bar(&self) -> Container<'_, Message> {
        let home = self.home_button();
        let back = self.back_button();
        let selector = self.select_button();
        let download = self.download_button();
        Container::new(row![selector, back, home, download].spacing(5.).wrap())
            .style(|theme| {
                let Palette { primary, .. } = theme.palette();
                container::Style {
                    border: Border {
                        width: 1.,
                        radius: Radius::new(20),
                        color: primary,
                    },
                    ..Default::default()
                }
            })
            .center_x(Length::Fill)
            .padding(12.)
    }

    fn download_button(&self) -> Column<'_, Message> {
        let ad = self.downloads.active_count();
        let active_downloads = (ad != 0).then_some(Text::new(ad));
        let msg: Message = if self.select.on && !self.select.units.is_empty() {
            DownloadMessage::QueueFromSelectedStart.into()
        } else {
            DownloadMessage::TogglePreview.into()
        };
        let button = svg_button(IconName::Download.get()).on_press(msg);
        iced::widget::column![button, active_downloads].align_x(Alignment::Center)
    }

    fn select_button(&self) -> Button<'_, Message> {
        svg_button(if self.select.on {
            IconName::Close.get()
        } else {
            IconName::Select.get()
        })
        .on_press(ClientMessage::ToggleSelectMode.into())
    }
    fn back_button(&self) -> Button<'_, Message> {
        let mut path = self.target.clone();
        let msg = path.pop().then_some(ClientMessage::GoToPath(path).into());
        Button::new("back").on_press_maybe(msg)
    }
    fn home_button(&self) -> Button<'_, Message> {
        if self.target == PathBuf::new() {
            go_home_button()
        } else {
            svg_button(IconName::Home.get())
                .on_press(ClientMessage::GoToPath(PathBuf::new()).into())
        }
    }
}

trait UnitViews {
    fn button<'a>(&'a self, selected: &'a Selected) -> MouseArea<'a, Message>;
}

impl UnitViews for Unit {
    fn button<'a>(&'a self, selected: &'a Selected) -> MouseArea<'a, Message> {
        let svg = svg_from_icon_data(self.icon());
        let text = Text::new(self.name());
        let row = row![svg, text].spacing(4.);
        mouse_area(Button::new(row).style(|theme, _| {
            let selected = selected.is_selected(self);
            let Palette {
                background,
                primary,
                text,
                ..
            } = theme.palette();
            Style {
                border: Border {
                    color: if selected { primary } else { background },
                    width: 1.,
                    radius: Radius::new(5.),
                },
                text_color: text,
                ..Default::default()
            }
        }))
        .interaction(Interaction::Pointer)
        .on_release(ClientMessage::UnitClick(self.clone()).into())
        .on_double_click(ClientMessage::UnitDoubleClick(self.clone()).into())
    }
}

fn svg_button<'a>(icon: &'a [u8]) -> Button<'a, Message> {
    Button::new(svg_from_icon_data(icon))
        .style(|_, _| Style {
            border: Border {
                width: 1.,
                radius: Radius::new(2.),
                ..Default::default()
            },
            ..Default::default()
        })
        .padding(7.)
}

impl State {
    pub fn handle_client_msg(&mut self, msg: ClientMessage) -> Task<Message> {
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
            ClientMessage::UnitDoubleClick(unit) => match (unit.kind, &state.grpc) {
                (UnitKind::Folder, Some(grpc)) => {
                    state.target = unit.path.clone();
                    Task::perform(grpc.clone().ls(unit.path.clone()), move |xs| {
                        ClientMessage::RefreshUnits(xs).into()
                    })
                }
                _ => {
                    println!("opening file {unit:#?} is not supported yet");
                    Task::none()
                }
            },
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
            ClientMessage::Download(msg) => self.handle_downloads_msg(msg),
        }
    }
}
