use std::net::{IpAddr, Ipv4Addr};

use assets::FOLDER_ICON;
use common::Unit;
use delivery::Delivery;
use iced::{
    Background, Border, Color, Length, Shadow, Task,
    theme::Palette,
    widget::{Button, Container, Svg, Text, column, row, scrollable, svg},
};

use crate::{Message, home::go_home_button, serve::Origin};

#[derive(Debug, Clone)]
pub enum ClientMessage {}

impl ClientMessage {
    pub fn handle(self, state: &mut ClientState) -> Task<Message> {
        Task::none()
    }
}

#[derive(Debug, Clone)]
pub struct ClientState {
    pub delivery: Delivery,
    pub units: Vec<Unit>,
}

impl Default for ClientState {
    fn default() -> Self {
        Self {
            delivery: Delivery::new(
                Origin {
                    ip: IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
                    port: 0,
                }
                .to_string(),
            ),
            units: Vec::new(),
        }
    }
}

impl ClientState {
    pub fn view(&self) -> Container<'_, Message> {
        let home = go_home_button();
        let col = self
            .units
            .iter()
            .fold(column![home].spacing(10.), |acc, x| acc.push(x.button()));
        Container::new(scrollable(col).width(Length::Fill))
    }
}

trait UnitViews {
    fn button(&self) -> Button<'_, Message>;
}

impl UnitViews for Unit {
    fn button(&self) -> Button<'_, Message> {
        let icon = Svg::new(svg::Handle::from_memory(FOLDER_ICON))
            .width(40.)
            .style(|_, _| svg::Style {
                color: Some(Color::WHITE),
            });
        let text = Text::new(self.name());
        let row = row![icon, text];
        Button::new(row).style(|_, _| iced::widget::button::Style {
            background: Some(Background::Color(iced::Color::BLACK)),
            text_color: Color::WHITE,
            border: Border::default(),
            shadow: Shadow::default(),
        })
    }
}
