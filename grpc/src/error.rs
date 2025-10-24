use std::{net::AddrParseError, sync::Arc};
use tonic::transport;

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
