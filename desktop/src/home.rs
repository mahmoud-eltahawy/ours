use std::net::{AddrParseError, IpAddr};

use crate::{Message, Page, svg_from_icon_data};
use common::assets::IconName;
use iced::{
    Alignment, Background, Border, Element, Length,
    border::Radius,
    theme::Palette,
    widget::{
        Button, Container, Text, button, center, column, container, mouse_area, opaque, row, stack,
        text_input::{self, Style},
    },
};

#[derive(Default)]
pub struct HomeState {
    pub show_form: bool,
    pub url_form: UrlForm,
}

#[derive(Default)]
pub struct UrlForm {
    pub ip: String,
    pub valid_ip: Option<IpAddr>,
    pub port: u16,
}

#[derive(Clone)]
pub enum HomeMessage {
    PortNewInput(Result<u16, std::num::ParseIntError>),
    IpNewInput {
        valid_ip: Result<IpAddr, AddrParseError>,
        input_value: String,
    },
    SubmitInput(IpAddr, u16),
    ToggleInputModal,
}

impl From<HomeMessage> for Message {
    fn from(value: HomeMessage) -> Self {
        Self::Home(value)
    }
}

impl HomeState {
    pub fn view<'a>(&'a self) -> Element<'a, Message> {
        let title = Text::new("Choose app mode").size(80).center();
        let server_button = self.go_to_server_button();
        let client_button = self.go_to_client_button();

        let main_content = column![title, server_button, client_button]
            .align_x(Alignment::Center)
            .spacing(20.);

        let main_content = Container::new(main_content)
            .center_x(Length::Fill)
            .center_y(Length::Fill);

        if !self.show_form {
            return main_content.into();
        }
        modal(
            main_content,
            self.url_form.view(),
            HomeMessage::ToggleInputModal.into(),
        )
    }

    pub fn go_to_server_button<'a>(&'a self) -> Button<'a, Message> {
        let content = Text::new("go to server mode").center().size(60);
        Button::new(content)
            .padding(30.)
            .style(move |theme, _| common_button_style(theme))
            .on_press(Message::GoToPage(Page::Server))
    }

    pub fn go_to_client_button<'a>(&'a self) -> Button<'a, Message> {
        let content = Text::new("go to client mode").center().size(60);
        Button::new(content)
            .padding(30.)
            .style(move |theme, _| common_button_style(theme))
            .on_press(HomeMessage::ToggleInputModal.into())
    }
}

fn style_input(theme: &iced::Theme) -> Style {
    let Palette {
        background,
        text,
        primary,
        success,
        ..
    } = theme.palette();
    Style {
        background: iced::Background::Color(background),
        border: Border {
            color: primary,
            width: 2.,
            radius: Radius {
                top_left: 8.,
                top_right: 8.,
                bottom_right: 8.,
                bottom_left: 8.,
            },
        },
        placeholder: text,
        value: text,
        selection: primary,
        icon: success,
    }
}

impl UrlForm {
    pub fn view(&self) -> Container<'_, Message> {
        let title = Text::new("choose client address").size(50.).center();
        let ip_input = self.ip_input();
        let port_input = self.port_input();
        let url_input = row![ip_input, port_input].spacing(10.);

        let submit = self.submit_button();
        let cancel = self.cancle_button();
        let buttons = row![submit, cancel].spacing(10.);

        let content = column![title, url_input, buttons]
            .align_x(Alignment::Center)
            .spacing(20.)
            .padding(20.);
        Container::new(content).style(form_style).padding(20.)
    }

    fn submit_button(&self) -> Button<'_, Message> {
        let content = Text::new("submit").size(60.).center();
        Button::new(content).on_press_maybe(
            self.valid_ip
                .map(|ip| HomeMessage::SubmitInput(ip, self.port).into())
                .take_if(|_| self.port != 0),
        )
    }

    fn port_input(&self) -> text_input::TextInput<'_, Message> {
        text_input::TextInput::new(
            "insert port",
            &if self.port != 0 {
                self.port.to_string()
            } else {
                "".to_string()
            },
        )
        .size(30.)
        .padding(10.)
        .align_x(Alignment::Center)
        .style(|theme, _| style_input(theme))
        .on_input(|x| HomeMessage::PortNewInput(x.parse::<u16>()).into())
    }

    fn ip_input(&self) -> text_input::TextInput<'_, Message> {
        text_input::TextInput::new("insert ip", &self.ip.to_string())
            .size(30.)
            .padding(10.)
            .align_x(Alignment::Center)
            .style(|theme, _| {
                if self.valid_ip.is_some() {
                    style_input(theme)
                } else {
                    Style {
                        value: theme.palette().danger,
                        ..style_input(theme)
                    }
                }
            })
            .on_input(|x| {
                HomeMessage::IpNewInput {
                    valid_ip: x.parse::<IpAddr>(),
                    input_value: x,
                }
                .into()
            })
    }

    fn cancle_button(&self) -> Button<'_, Message> {
        let cancel = Text::new("cancel").size(60.).center();
        Button::new(cancel).on_press(HomeMessage::ToggleInputModal.into())
    }
}

fn form_style(theme: &iced::Theme) -> container::Style {
    container::Style {
        border: Border {
            width: 12.,
            radius: Radius::new(22.),
            color: theme.palette().primary,
        },
        ..Default::default()
    }
}

fn common_button_style(theme: &iced::Theme) -> button::Style {
    button::Style {
        border: iced::Border {
            width: 5.,
            radius: Radius::new(30.),
            color: theme.palette().text,
        },
        text_color: theme.palette().text,
        ..Default::default()
    }
}

pub fn modal<'a, Message>(
    base: impl Into<Element<'a, Message>>,
    content: impl Into<Element<'a, Message>>,
    on_blur: Message,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    stack![
        base.into(),
        opaque(
            mouse_area(center(opaque(content)).style(|theme| {
                let mut bg = theme.palette().background;
                bg.a = 0.8;
                container::Style {
                    background: Some(Background::Color(bg)),
                    ..container::Style::default()
                }
            }))
            .on_press(on_blur)
        )
    ]
    .into()
}

pub fn go_home_button<'a>() -> Button<'a, Message> {
    Button::new(svg_from_icon_data(IconName::Home.get()))
        .on_press(Message::GoToPage(Page::Home))
        .style(|theme, _| button::Style {
            background: Some(Background::Color(theme.palette().danger)),
            ..Default::default()
        })
}
