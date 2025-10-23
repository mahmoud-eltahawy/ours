use std::path::PathBuf;

use crate::{Message, State, main_window::MainWindowMessage};
use iced::{Element, widget::Text};

pub struct ClientState {
    grpc: grpc::client::RpcClient,
    target: PathBuf,
    pub units: Vec<grpc::client::Unit>,
}

impl ClientState {
    pub fn new(grpc: grpc::client::RpcClient) -> Self {
        Self {
            grpc,
            target: PathBuf::new(),
            units: Vec::new(),
        }
    }
}

#[derive(Clone)]
pub enum ClientMessage {
    RefreshUnits(Vec<grpc::client::Unit>),
    PrepareGrpc(Option<grpc::client::RpcClient>),
}

impl From<ClientMessage> for Message {
    fn from(value: ClientMessage) -> Self {
        Message::MainWindow(MainWindowMessage::Client(value))
    }
}

impl State {
    pub fn client<'a>(&'a self) -> Element<'a, Message> {
        Text::new("client page").into()
    }
}
