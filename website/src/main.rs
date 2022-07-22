mod errors;
mod options;
mod static_content;

pub use errors::Error;

use std::{error::Error as StdError, future::Future};

use axum::{response::Html, routing, Extension, Router};
use clap::Parser;
use tokio::{
    select,
    signal::unix::{signal, SignalKind},
};

use options::Options;

static INDEX: &str = include_str!("../html/index.html");

fn make_server(opts: &'static Options) -> impl Future {
    let app = Router::new()
        .route("/", routing::get(|| async { Html(INDEX) }))
        .route("/static/*path", routing::get(static_content::handler))
        .layer(Extension(opts));
    // .fallback(errors::not_found.into_service());
    axum::Server::bind(&opts.listen_addr).serve(app.into_make_service())
}

// Single-threaded runtime because we're not expecting much load
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn StdError>> {
    let opts = Options::parse();
    let opts = Box::leak(Box::new(opts));

    let mut sigint = signal(SignalKind::interrupt())?;
    let mut sigterm = signal(SignalKind::terminate())?;
    let server = make_server(opts);

    let signal: &str;

    select! {
        _ = sigint.recv() => signal = "INT",
        _ = sigterm.recv() => signal = "TERM",
        _ = server => unreachable!(),
    };

    println!("Got SIG{signal}, exiting...");

    Ok(())
}
