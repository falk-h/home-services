use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};

use pulldown_cmark::{Options, Parser};
use tokio::fs;

use crate::util;

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
