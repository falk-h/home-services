mod errors;
mod options;
mod rendering;
mod services;

use std::{error::Error as StdError, io, net::SocketAddr, sync::Arc};

use axum::{
    handler::HandlerWithoutStateExt,
    response::{IntoResponse, Redirect},
    routing, Extension, Router,
};
use clap::Parser;
use tera::Context;
use tokio::{
    select,
    signal::unix::{signal, SignalKind},
};
use tower::util::MapErr;
use tower_http::{
    services::ServeDir,
    trace::{DefaultOnRequest, DefaultOnResponse, TraceLayer},
};
use tracing::Level;
use tracing_subscriber::filter::EnvFilter;
use trust_dns_resolver::{
    name_server::{GenericConnection, GenericConnectionProvider, TokioRuntime},
    AsyncResolver,
};

use errors::Error;
use options::Options;
use rendering::Renderer;
use services::StatusChecker;

// Single-threaded runtime because we're not expecting much load
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn StdError>> {
    // Docker-compose can handle timestamps for us.
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .without_time()
        .init();

    let opts = Options::parse();

    let template_dir = opts.template_dir.to_str().unwrap_or_else(|| {
        panic!(
            "Path to template directory {:?} contains invalid UTF-8",
            opts.template_dir,
        )
    });
    let renderer = Renderer::new(template_dir)
        .map_err(|e| tracing::error!("{e}"))
        .unwrap();
    let renderer = Arc::new(renderer);

    Error::init_html_rendering(renderer.clone());

    // Type shenanigans because ServeDir *really* wants a fallback service with
    // Error = io::Error, while Router *really* wants its inner services to have
    // Error = Infallible.
    let redir_to_index_io = MapErr::new(redir_to_index.into_service(), |_| -> io::Error {
        // SAFETY: The sole parameter to this closure is Infallible, so this
        // closure can never be called.
        unsafe { std::hint::unreachable_unchecked() }
    });

    let static_file_handler = ServeDir::new(&opts.static_dir)
        .precompressed_br()
        .call_fallback_on_method_not_allowed(true)
        .fallback(redir_to_index_io);

    let service_checker = Arc::new(StatusChecker::new().await);

    // let foo: &dyn Send = &index(Extension(service_checker), Extension(renderer));

    let tracer = TraceLayer::new_for_http()
        .on_request(DefaultOnRequest::new().level(Level::DEBUG))
        .on_response(DefaultOnResponse::new().level(Level::INFO));

    let app = Router::new()
        .nest(
            "/static",
            routing::get_service(static_file_handler).handle_error(|e| async { Error::from(e) }),
        )
        .route("/", routing::get(index))
        .route("/ping", routing::get(ping))
        .route("/favicon.svg", routing::get(icon))
        .fallback(redir_to_index)
        .layer(Extension(service_checker))
        .layer(Extension(renderer))
        .layer(tracer);

    let server = axum::Server::bind(&opts.listen_addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>());

    tracing::info!("Listening on http://{}/", opts.listen_addr);

    let mut sigint = signal(SignalKind::interrupt())?;
    let mut sigterm = signal(SignalKind::terminate())?;

    let signal = select! {
        _ = sigint.recv() => "INT",
        _ = sigterm.recv() => "TERM",
        _ = server => unreachable!(),
    };

    println!("Got SIG{signal}, exiting...");

    Ok(())
}

type ArcExt<T> = Extension<Arc<T>>;
type Resolver = AsyncResolver<GenericConnection, GenericConnectionProvider<TokioRuntime>>;

async fn index(
    checker: ArcExt<StatusChecker>,
    renderer: ArcExt<Renderer>,
) -> Result<impl IntoResponse, Error> {
    let services = services::check_statuses(&checker.0).await;
    tracing::debug!("Got service data {services:?}");

    let mut context = Context::new();
    context.insert("services", &services);

    renderer.render_html("index.html", &context)
}

async fn ping() -> impl IntoResponse {
    "pong"
}

async fn icon(renderer: ArcExt<Renderer>) -> Result<impl IntoResponse, Error> {
    let icon = renderer.render("favicon.svg", &Context::new())?;
    Ok((
        [
            ("content-type", "image/svg+xml"),
            ("cache-control", "no-store"), // Doesn't work in Firefox :(
        ],
        icon,
    ))
}

async fn redir_to_index() -> impl IntoResponse {
    Redirect::temporary("/")
}
