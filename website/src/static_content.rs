//! Handler for static content.

use std::path::PathBuf;

use axum::{
    extract::Path as PathExt,
    http::HeaderValue,
    response::{IntoResponse, Response},
    Extension,
};
use tokio::fs;

use crate::{options::Options, util, Error};

pub async fn handler(
    opts: Extension<&Options>,
    PathExt(path): PathExt<PathBuf>,
) -> Result<Response, Error> {
    let file = util::join_absolute_paths(&opts.static_dir, path)?;
    let body = fs::read_to_string(&file).await?;
    let mut response = body.into_response();

    if let Some(Ok(t)) = mime_guess::from_path(file)
        .first_raw()
        .map(HeaderValue::from_str)
    {
        response.headers_mut().insert("content-type", t);
    }

    Ok(response)
}
