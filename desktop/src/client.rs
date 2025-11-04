use crate::home::go_home_button;
use crate::{Message, svg_from_icon_data};
use common::assets::IconName;
use grpc::client::RpcClient;
use grpc::error::RpcError;
use grpc::top::{Selected, Unit};
use iced::theme::Palette;
use iced::widget::container;
use iced::{
    Border, Element, Length,
    border::Radius,
    mouse::Interaction,
    widget::{Button, Container, MouseArea, Row, Text, button::Style, mouse_area, row, scrollable},
};
use std::path::PathBuf;

#[derive(Default)]
pub struct ClientState {
    pub grpc: Option<RpcClient>,
    pub target: PathBuf,
    pub select: Selected,
    pub units: Vec<grpc::top::Unit>,
}

impl ClientState {
    pub fn new(grpc: RpcClient) -> Self {
        Self {
            grpc: Some(grpc),
            target: PathBuf::new(),
            units: Vec::new(),
            select: Selected::default(),
        }
    }
}

#[derive(Clone)]
pub enum ClientMessage {
    RefreshUnits(Result<Vec<grpc::top::Unit>, RpcError>),
    PrepareGrpc(Result<RpcClient, RpcError>),
    UnitClick(Unit),
    UnitDoubleClick(Unit),
    QueueDownloadFromSelected,
    ToggleSelectMode,
    GoToPath(PathBuf),
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
        Container::new(all)
            .padding(10.)
            .center_x(Length::Fill)
            .into()
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

    fn download_button(&self) -> Button<'_, Message> {
        svg_button(IconName::Download.get())
            .on_press(ClientMessage::QueueDownloadFromSelected.into())
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
                ..
            } = theme.palette();
            Style {
                border: Border {
                    color: if selected { primary } else { background },
                    width: 1.,
                    radius: Radius::new(5.),
                },
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
