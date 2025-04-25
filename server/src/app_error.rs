use axum::{
    Json,
    response::{self, IntoResponse},
};
use serde::Serialize;
use std::{fmt::Display, path::PathBuf};

#[derive(Debug, Serialize)]
pub enum AppError {
    FfmpagSpawn(PathBuf),
    FfmpagWait(PathBuf),
    FfmpagJoin,
    OtherError,
}

impl Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let sj = serde_json::json!(self);
        write!(f, "{}", sj)
    }
}

impl std::error::Error for AppError {}

impl IntoResponse for AppError {
    fn into_response(self) -> response::Response {
        Json(self).into_response()
    }
}
