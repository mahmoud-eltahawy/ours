use std::path::PathBuf;

use crate::{Message, main_window::MainWindowMessage, svg_from_icon_data};
use grpc::RpcError;
use grpc::client::RpcClient;
use grpc::top::{Selected, Unit};
use iced::{
    Border, Color, Element, Length,
    border::Radius,
    mouse::Interaction,
    widget::{Button, Container, MouseArea, Row, Text, button::Style, mouse_area, row, scrollable},
};

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
}

impl From<ClientMessage> for Message {
    fn from(value: ClientMessage) -> Self {
        Message::MainWindow(MainWindowMessage::Client(value))
    }
}

impl ClientState {
    pub fn view<'a>(&'a self) -> Element<'a, Message> {
        self.units().into()
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
            .style(|_| iced::widget::container::Style {
                border: Border {
                    color: Color::WHITE,
                    width: 2.,
                    radius: Radius::new(20),
                },
                ..Default::default()
            })
            .padding(10.);
        scrollable(units).width(Length::Fill)
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
        mouse_area(Button::new(row).style(|_, _| {
            let selected = selected.is_selected(self);
            Style {
                border: Border {
                    color: if selected { Color::WHITE } else { Color::BLACK },
                    width: 5.,
                    radius: Radius::new(5.),
                },
                text_color: Color::WHITE,
                ..Default::default()
            }
        }))
        .interaction(Interaction::Pointer)
        .on_release(ClientMessage::UnitClick(self.clone()).into())
        .on_double_click(ClientMessage::UnitDoubleClick(self.clone()).into())
    }
}
