//! Error handling.

use std::{
    io::{self, ErrorKind},
    path::StripPrefixError,
    sync::Arc,
};

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use once_cell::sync::OnceCell;
use thiserror::Error;

use crate::rendering::Renderer;

#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("Path manipulation error: {0}")]
    PathManip(#[from] StripPrefixError),

    #[error("Templating error: {0}")]
    TemplateError(#[from] tera::Error),
}

static RENDERER: OnceCell<Arc<Renderer>> = OnceCell::new();

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        if let Self::Io(err) = &self {
            if err.kind() == ErrorKind::NotFound {
                return (StatusCode::NOT_FOUND, "not found :(").into_response();
            }
        }

        let status = StatusCode::INTERNAL_SERVER_ERROR;

        match RENDERER.get().map(|renderer| renderer.render_error(&self)) {
            Some(Ok(html)) => (status, html).into_response(),
            Some(Err(s)) => (status, s).into_response(),
            None => (
                status,
                format!(
                    "Error rendering was not initialized while rendering the following error: \
                    {self}"
                ),
            )
                .into_response(),
        }
    }
}

impl Error {
    pub fn init_html_rendering(renderer: Arc<Renderer>) {
        RENDERER.set(renderer).ok();
    }
}
