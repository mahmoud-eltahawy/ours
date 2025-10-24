pub mod client;
pub mod server;
pub mod top;
mod nav {
    use tonic::include_proto;

    include_proto!("nav.v1");
}

pub use nav::UnitKind;
use std::{net::AddrParseError, sync::Arc};
pub use tonic::transport;

#[derive(Debug, Clone)]
pub enum RpcError {
    AddrParse(AddrParseError),
    Tonic(Arc<transport::Error>),
    TonicStatus(tonic::Status),
}

impl From<AddrParseError> for RpcError {
    fn from(value: AddrParseError) -> Self {
        Self::AddrParse(value)
    }
}

impl From<transport::Error> for RpcError {
    fn from(value: transport::Error) -> Self {
        Self::Tonic(Arc::new(value))
    }
}

impl From<tonic::Status> for RpcError {
    fn from(value: tonic::Status) -> Self {
        Self::TonicStatus(value)
    }
}
