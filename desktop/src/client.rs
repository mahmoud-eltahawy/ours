use crate::{Page, client::downloads::Downloads, home::go_home_button, svg_from_icon_data};
use common::assets::IconName;
use grpc::{
    UnitKind,
    client::RpcClient,
    error::RpcError,
    top::{Selected, Unit},
};
use iced::{
    Alignment, Border, Element, Length, Task,
    border::Radius,
    mouse::Interaction,
    theme::Palette,
    widget::{
        Button, Column, Container, MouseArea, Row, Text, button::Style, container, mouse_area, row,
        scrollable,
    },
};
use std::path::PathBuf;

pub mod downloads;

#[derive(Clone)]
pub struct State {
    pub grpc: RpcClient,
    pub target: PathBuf,
    pub select: Selected,
    pub units: Vec<Unit>,
}

impl State {
    pub fn new(grpc: RpcClient) -> Self {
        Self {
            grpc,
            target: PathBuf::new(),
            units: Vec::new(),
            select: Selected::default(),
        }
    }
}

#[derive(Clone)]
pub enum Message {
    RefreshUnits(Result<Vec<Unit>, RpcError>),
    UnitClick(Unit),
    UnitDoubleClick(Unit),
    ToggleSelectMode,
    GoToPath(PathBuf),
    Download(downloads::Message),
}

impl From<Message> for crate::Message {
    fn from(value: Message) -> Self {
        crate::Message::Client(value)
    }
}

impl State {
    pub fn view<'a>(&'a self, downloads: &Downloads) -> Element<'a, crate::Message> {
        let tools = self.tools_bar(downloads);
        let units = self.units();
        let all = iced::widget::column![tools, units]
            .spacing(10.)
            .width(Length::Fill);
        Container::new(all)
            .padding(10.)
            .center_x(Length::Fill)
            .into()
    }

    fn units(&self) -> scrollable::Scrollable<'_, crate::Message> {
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
            .height(Length::Fill)
            .width(Length::Fill)
            .padding(10.);
        scrollable(units).height(Length::Fill).width(Length::Fill)
    }

    fn tools_bar(&self, downloads: &Downloads) -> Container<'_, crate::Message> {
        let home = self.home_button();
        let back = self.back_button();
        let selector = self.select_button();
        let download = self.download_button(downloads);
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

    fn download_button(&self, downloads: &Downloads) -> Column<'_, crate::Message> {
        let ad = downloads.active_count();
        let active_downloads = (ad != 0).then_some(Text::new(ad));
        let msg: crate::Message = if self.select.on && !self.select.units.is_empty() {
            downloads::Message::QueueFromSelectedStart.into()
        } else {
            downloads::Message::TogglePreview.into()
        };
        let button = svg_button(IconName::Download.get()).on_press(msg);
        iced::widget::column![button, active_downloads].align_x(Alignment::Center)
    }

    fn select_button(&self) -> Button<'_, crate::Message> {
        svg_button(if self.select.on {
            IconName::Close.get()
        } else {
            IconName::Select.get()
        })
        .on_press(Message::ToggleSelectMode.into())
    }
    fn back_button(&self) -> Button<'_, crate::Message> {
        let mut path = self.target.clone();
        let msg = path.pop().then_some(Message::GoToPath(path).into());
        Button::new("back").on_press_maybe(msg)
    }
    fn home_button(&self) -> Button<'_, crate::Message> {
        if self.target == PathBuf::new() {
            go_home_button()
        } else {
            svg_button(IconName::Home.get()).on_press(Message::GoToPath(PathBuf::new()).into())
        }
    }
}

trait UnitViews {
    fn button<'a>(&'a self, selected: &'a Selected) -> MouseArea<'a, crate::Message>;
}

impl UnitViews for Unit {
    fn button<'a>(&'a self, selected: &'a Selected) -> MouseArea<'a, crate::Message> {
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
        .on_release(Message::UnitClick(self.clone()).into())
        .on_double_click(Message::UnitDoubleClick(self.clone()).into())
    }
}

fn svg_button<'a>(icon: &'a [u8]) -> Button<'a, crate::Message> {
    Button::new(svg_from_icon_data(icon))
        .style(|_, _| Style {
            border: Border {
                width: 1.,
                radius: Radius::new(2.),
                ..Default::default()
            },
            background: None,
            ..Default::default()
        })
        .padding(7.)
}

impl crate::State {
    pub fn handle_client_msg(&mut self, msg: Message) -> Task<crate::Message> {
        let Page::Client(state) = &mut self.page else {
            unreachable!()
        };
        match msg {
            Message::RefreshUnits(units) => {
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
            Message::UnitClick(unit) => {
                if state.select.on {
                    state.select.toggle_unit_selection(&unit);
                } else {
                    state.select.toggle_unit_alone_selection(&unit);
                }
                Task::none()
            }
            Message::UnitDoubleClick(unit) => match unit.kind {
                UnitKind::Folder => {
                    state.target = unit.path.clone();
                    Task::perform(state.grpc.clone().ls(unit.path.clone()), move |xs| {
                        Message::RefreshUnits(xs).into()
                    })
                }
                _ => {
                    println!("opening file {unit:#?} is not supported yet");
                    Task::none()
                }
            },
            Message::ToggleSelectMode => {
                if state.select.on {
                    state.select.clear();
                } else {
                    state.select.on = true;
                }
                Task::none()
            }
            Message::GoToPath(path) => {
                state.target = path.clone();
                Task::perform(state.grpc.clone().ls(path), |xs| {
                    Message::RefreshUnits(xs).into()
                })
            }
            Message::Download(msg) => {
                let grpc = state.grpc.clone();
                self.handle_downloads_msg(msg, grpc)
            }
        }
    }
}
