use std::path::{Path, PathBuf};

use axum::{
    http::Uri,
    response::{Html, IntoResponse, Response},
    Extension,
};

use crate::{shared_state::State, Error};

fn adjust_path(path: &mut PathBuf) {
    if path == Path::new("/") {
        path.push("index");
    }
    path.set_extension("md");
}

pub async fn handler(state: Extension<&State>, uri: Uri) -> Result<Response, Error> {
    let mut path = PathBuf::from(uri.path());
    adjust_path(&mut path);
    let mut renderer = state.md_renderer.lock().await;
    let html = renderer.render(path).await?;
    Ok(Html(html.into_owned()).into_response())
}
