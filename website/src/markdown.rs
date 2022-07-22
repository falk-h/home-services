use std::{
    borrow::Cow,
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use axum::{
    http::Uri,
    response::{Html, IntoResponse, Response},
    Extension,
};
use handlebars::Handlebars;
use pulldown_cmark::{Event, Options, Parser, Tag};
use tokio::fs;

use crate::{shared_state::State, util, Error};

const BASE_TEMPLATE: &str = "base.hbs";

#[derive(Debug)]
pub struct MdRenderer {
    base_dir: PathBuf,
    base_template: PathBuf,
    templater: Handlebars<'static>,
}

impl MdRenderer {
    pub async fn new<P, Q>(base_dir: P, template_dir: Q) -> Self
    where
        P: AsRef<Path>,
        Q: AsRef<Path>,
    {
        let base_dir = base_dir.as_ref().to_owned();
        assert!(base_dir.is_dir());

        let template_dir = template_dir.as_ref().to_owned();
        assert!(template_dir.is_dir());
        let base_template = template_dir.join(BASE_TEMPLATE);
        assert!(base_template.exists());

        Self {
            base_dir,
            base_template,
            templater: Handlebars::new(),
        }
    }

    pub async fn render<P: AsRef<Path>>(&mut self, file: P) -> Result<Cow<'_, str>, crate::Error> {
        let md_file = util::join_absolute_paths(&self.base_dir, file)?;

        let (md, template) = tokio::join!(
            fs::read_to_string(md_file),
            fs::read_to_string(&self.base_template),
        );
        let md = md?;

        let data = md_to_data(&md);
        let rendered = self.templater.render_template(&template?, &data)?;
        let buf = minify_html::minify(rendered.as_bytes(), &Default::default());
        let html = String::from_utf8(buf).unwrap();

        Ok(Cow::Owned(html))
    }
}

fn md_to_data(md: &str) -> BTreeMap<&str, String> {
    BTreeMap::from([("title", extract_title(md)), ("body", md_to_string(md))])
}

fn adjust_path(path: &mut PathBuf) {
    if path == Path::new("/") {
        path.push("index");
    }
    path.set_extension("md");
}

fn extract_title(md: &str) -> String {
    let mut parsed = Parser::new_ext(&md, Options::all());

    // Continue to the first heading.
    parsed.find(|event| matches!(event, Event::Start(Tag::Heading(_, _, _))));

    match parsed.next() {
        Some(Event::Text(s)) => s.to_string(),
        _ => "".to_owned(),
    }
}

fn md_to_string(md: &str) -> String {
    let parser = Parser::new_ext(&md, Options::all());
    let mut ret = String::with_capacity(md.len());
    pulldown_cmark::html::push_html(&mut ret, parser);
    ret
}

pub async fn handler(state: Extension<&State>, uri: Uri) -> Result<Response, Error> {
    let mut path = PathBuf::from(uri.path());
    adjust_path(&mut path);
    let mut renderer = state.md_renderer.lock().await;
    let html = renderer.render(path).await?;
    Ok(Html(html.into_owned()).into_response())
}
