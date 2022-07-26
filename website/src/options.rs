use std::{net::SocketAddr, path::PathBuf};

use clap::Parser;

#[derive(Clone, Debug, Parser)]
pub struct Options {
    /// Socket address to listen on.
    #[clap(short, long, env, default_value = "0.0.0.0:80")]
    pub listen_addr: SocketAddr,

    /// Directory with static files.
    #[clap(short, long, env, default_value = "static")]
    pub static_dir: PathBuf,

    /// Directory with misc. templates.
    #[clap(short, long, env, default_value = "templates")]
    pub template_dir: PathBuf,
}
