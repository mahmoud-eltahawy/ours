use iced::{
    Color, Element, Subscription, Task, exit,
    theme::Style,
    window::{self, Settings},
};

mod download_window;
mod main_window;

fn main() {
    iced::daemon(State::new, State::update, State::view)
        .subscription(State::close_event)
        .title(State::title)
        .style(|_, _| Style {
            background_color: Color::BLACK,
            text_color: Color::WHITE,
        })
        .run()
        .unwrap();
}

struct State {
    main_window_id: window::Id,
    download_window_id: Option<window::Id>,
    main_window_page: main_window::Page,
    pub main_window_state: main_window::MainWindowState,
}

#[derive(Clone)]
enum Message {
    WindowOpened(window::Id),
    WindowClosed(window::Id),
    GoToPage(main_window::Page),
    MainWindow(main_window::MainWindowMessage),
}

impl State {
    fn title(&self, _: window::Id) -> String {
        String::from("ours")
    }
    fn new() -> (Self, Task<Message>) {
        let (id, task) = window::open(Settings::default());
        (
            State {
                main_window_id: id,
                download_window_id: None,
                main_window_page: Default::default(),
                main_window_state: Default::default(),
            },
            task.map(Message::WindowOpened),
        )
    }
    fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::WindowOpened(id) => {
                if self.main_window_id != id {
                    self.download_window_id = Some(id);
                }
                Task::none()
            }
            Message::WindowClosed(id) => {
                if self.download_window_id.is_some_and(|x| x == id) {
                    self.download_window_id = None;
                    Task::none()
                } else {
                    exit()
                }
            }
            Message::GoToPage(page) => {
                self.main_window_page = page;
                Task::none()
            }
            Message::MainWindow(main_window_message) => match main_window_message {
                main_window::MainWindowMessage::Home(home_message) => match home_message {
                    main_window::home::HomeMessage::PortNewInput(port) => {
                        self.main_window_state.home.url_form.port = port;
                        Task::none()
                    }
                    main_window::home::HomeMessage::IpNewInput {
                        valid_ip,
                        input_value,
                    } => {
                        self.main_window_state.home.url_form.ip = input_value;
                        self.main_window_state.home.url_form.valid_ip = valid_ip;
                        Task::none()
                    }
                    main_window::home::HomeMessage::SubmitInput(ip_addr, port) => todo!(),
                    main_window::home::HomeMessage::ToggleInputModal => {
                        self.main_window_state.home.show_form =
                            !self.main_window_state.home.show_form;
                        Task::none()
                    }
                },
            },
        }
    }

    fn view<'a>(&'a self, window_id: window::Id) -> Element<'a, Message> {
        if self.main_window_id == window_id {
            self.main_window_view()
        } else if self.download_window_id.is_some_and(|x| x == window_id) {
            self.download_window_view()
        } else {
            unreachable!()
        }
    }

    fn close_event(&self) -> Subscription<Message> {
        window::close_events().map(Message::WindowClosed)
    }
}
