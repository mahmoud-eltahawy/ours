pub mod client;
pub mod server;
mod nav {
    use tonic::include_proto;

    include_proto!("nav.v1");
}
use std::net::AddrParseError;

pub use nav::UnitKind;
pub use tonic::transport;

#[derive(Debug)]
pub enum RpcError {
    AddrParse(AddrParseError),
    Tonic(transport::Error),
    TonicStatus(tonic::Status),
}

impl From<AddrParseError> for RpcError {
    fn from(value: AddrParseError) -> Self {
        Self::AddrParse(value)
    }
}

impl From<transport::Error> for RpcError {
    fn from(value: transport::Error) -> Self {
        Self::Tonic(value)
    }
}

impl From<tonic::Status> for RpcError {
    fn from(value: tonic::Status) -> Self {
        Self::TonicStatus(value)
    }
}
