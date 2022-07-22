use tokio::sync::Mutex;

use crate::markdown::MdRenderer;
use crate::options::Options;

#[derive(Debug)]
pub struct State {
    pub md_renderer: Mutex<MdRenderer>,
}

impl State {
    pub fn new(opts: &Options) -> Self {
        Self {
            md_renderer: Mutex::new(MdRenderer::new(&opts.markdown_dir)),
        }
    }
}
