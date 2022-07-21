use std::{error::Error, future::Future, net::SocketAddr};

use axum::{response::Html, routing, Router};
use clap::Parser;
use tokio::{
    select,
    signal::unix::{signal, SignalKind},
};

#[derive(Debug, Parser)]
struct Args {
    /// Socket address to listen on.
    #[clap(short, long, env, default_value = "0.0.0.0:80")]
    listen_addr: SocketAddr,
}

static INDEX: &str = include_str!("../html/index.html");

fn make_server(args: &Args) -> impl Future {
    let app = Router::new().route("/", routing::get(|| async { Html(INDEX) }));
    axum::Server::bind(&args.listen_addr).serve(app.into_make_service())
}

// Single-threaded runtime because we're not expecting much load
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let mut sigint = signal(SignalKind::interrupt())?;
    let mut sigterm = signal(SignalKind::terminate())?;
    let server = make_server(&args);

    let signal: &str;

    select! {
        _ = sigint.recv() => signal = "INT",
        _ = sigterm.recv() => signal = "TERM",
        _ = server => unreachable!(),
    };

    println!("Got SIG{signal}, exiting...");

    Ok(())
}
