use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};

use axum::{
    http::Uri,
    response::{Html, IntoResponse, Response},
    Extension,
};
use pulldown_cmark::{Options, Parser};
use tokio::fs;

use crate::{shared_state::State, util, Error};

#[derive(Debug)]
pub struct MdRenderer {
    base_dir: PathBuf,
}

impl MdRenderer {
    pub fn new<P: AsRef<Path>>(base_dir: P) -> Self {
        Self {
            base_dir: base_dir.as_ref().to_owned(),
        }
    }

    pub async fn render<P: AsRef<Path>>(&mut self, file: P) -> Result<Cow<'_, str>, crate::Error> {
        let file = util::join_absolute_paths(&self.base_dir, file)?;
        let md = fs::read_to_string(file).await?;
        let parser = Parser::new_ext(&md, Options::all());
        let mut ret = String::with_capacity(md.len());
        pulldown_cmark::html::push_html(&mut ret, parser);
        Ok(Cow::Owned(ret))
    }
}

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
