use axum::{
    body::Body,
    http::{Request, StatusCode},
    routing::get,
    Router,
};
use tracing_subscriber::{
    filter::Targets, fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter,
};

use std::net::SocketAddr;
use tower_http::trace::{self, TraceLayer};
use tracing::{error, info_span, instrument, Level, Span};

#[tokio::main]
async fn main() {
    // tracing_subscriber::fmt().with_target(false).json().init();

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(fmt::layer().pretty())
        .init();

    let app = Router::new().route("/", get(hello_world)).layer(
        TraceLayer::new_for_http()
            .make_span_with(make_span)
            .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
    );

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    tracing::info!("listening on {}", addr);
    axum::serve(listener, app).await.unwrap();
}

#[instrument]
async fn hello_world() -> &'static str {
    error!("uh oh!");
    "Hello World!"
}

fn make_span(request: &Request<Body>) -> Span {
    info_span!("request", method = %request.method(), uri = %request.uri(), version = ?request.version(), headers = ?request.headers())
}
