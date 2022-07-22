use std::{net::SocketAddr, path::PathBuf};

use clap::Parser;

#[derive(Clone, Debug, Parser)]
pub struct Options {
    /// Socket address to listen on.
    #[clap(short, long, env, default_value = "0.0.0.0:80")]
    pub listen_addr: SocketAddr,

    /// Directory with static files.
    #[clap(short, long, env, default_value = "./static")]
    pub static_dir: PathBuf,

    /// Directory with Markdown content.
    #[clap(short, long, env, default_value = "./md")]
    pub markdown_dir: PathBuf,
}
