//! Handler for static content.

use std::path::{Component, Path, PathBuf};

use axum::{
    extract::Path as PathExt,
    http::HeaderValue,
    response::{IntoResponse, Response},
    Extension,
};
use tokio::fs;

use crate::{options::Options, Error};

pub async fn handler(
    opts: Extension<&Options>,
    PathExt(path): PathExt<PathBuf>,
) -> Result<Response, Error> {
    let mut path: &Path = &path;

    // The Path extractor extracts the path including a leading `/`. This breaks
    // when concatenating it with the static dir, so make it relative. See the
    // docs for `Path.join`.
    if path.is_absolute() {
        path = path.strip_prefix(Component::RootDir)?;
        debug_assert!(path.is_relative());
    }

    let file = opts.static_dir.join(path);
    dbg!(&file);
    let body = fs::read_to_string(&file).await?;
    dbg!(&body);
    let mut response = body.into_response();

    if let Some(Ok(t)) = mime_guess::from_path(file)
        .first_raw()
        .map(HeaderValue::from_str)
    {
        response.headers_mut().insert("content-type", t);
    }

    Ok(response)
}
