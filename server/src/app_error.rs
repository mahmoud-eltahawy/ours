use axum::{
    Json,
    extract::multipart::MultipartError,
    response::{self, IntoResponse},
};
use serde::Serialize;
use std::{
    fmt::Display,
    path::{PathBuf, StripPrefixError},
};
use tokio::{io, task::JoinError};

pub type ServerResult<T> = std::result::Result<T, ServerError>;

#[derive(Debug, Serialize)]
pub enum ServerError {
    FfmpagSpawn(PathBuf),
    FfmpagWait(PathBuf),
    Join(String),
    Io(String),
    Copy,
    NonePort,
    NonePathFilename,
    MultiPart(String),
    StripPrefixError,
}

impl From<JoinError> for ServerError {
    fn from(value: JoinError) -> Self {
        Self::Join(value.to_string())
    }
}

impl From<io::Error> for ServerError {
    fn from(value: io::Error) -> Self {
        Self::Io(value.to_string())
    }
}

impl From<MultipartError> for ServerError {
    fn from(value: MultipartError) -> Self {
        Self::MultiPart(value.to_string())
    }
}

impl From<StripPrefixError> for ServerError {
    fn from(_value: StripPrefixError) -> Self {
        Self::StripPrefixError
    }
}

impl Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let sj = serde_json::json!(self);
        write!(f, "{}", sj)
    }
}

impl std::error::Error for ServerError {}

impl IntoResponse for ServerError {
    fn into_response(self) -> response::Response {
        Json(self).into_response()
    }
}
