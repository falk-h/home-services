//! Error handling.

use std::{
    io::{self, ErrorKind},
    path::StripPrefixError,
};

use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O error")]
    Io(#[from] io::Error),

    #[error("Path manipulation error")]
    PathManip(#[from] StripPrefixError),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status, msg) = match self {
            Self::Io(err) if err.kind() == ErrorKind::NotFound => {
                (StatusCode::NOT_FOUND, Html("not found :(".to_owned()))
            }
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(format!("Internal server error: {}", self)),
            ),
        };

        (status, msg).into_response()
    }
}
