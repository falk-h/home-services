mod errors;
pub use errors::Error;
mod markdown;
mod options;
mod shared_state;
mod static_content;
mod util;

use std::{error::Error as StdError, future::Future};

use axum::{handler::Handler, routing, Extension, Router};
use clap::Parser;
use tokio::{
    select,
    signal::unix::{signal, SignalKind},
};
use tower::ServiceBuilder;

use options::Options;
use shared_state::State;

fn make_server(opts: &'static Options, shared_state: &'static State) -> impl Future {
    let app = Router::new()
        .route("/static/*path", routing::get(static_content::handler))
        .layer(
            ServiceBuilder::new()
                .layer(Extension(shared_state))
                .layer(Extension(opts)),
        )
        .fallback(
            ServiceBuilder::new()
                .layer(Extension(shared_state))
                .service(markdown::handler.into_service()),
        );
    axum::Server::bind(&opts.listen_addr).serve(app.into_make_service())
}

// Single-threaded runtime because we're not expecting much load
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn StdError>> {
    let opts = Options::parse();
    let state = Box::leak(Box::new(State::new(&opts)));
    let opts = Box::leak(Box::new(opts));

    let mut sigint = signal(SignalKind::interrupt())?;
    let mut sigterm = signal(SignalKind::terminate())?;

    let server = make_server(opts, state);

    let signal: &str;

    select! {
        _ = sigint.recv() => signal = "INT",
        _ = sigterm.recv() => signal = "TERM",
        _ = server => unreachable!(),
    };

    println!("Got SIG{signal}, exiting...");

    Ok(())
}
