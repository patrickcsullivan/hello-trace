mod extractor;

use axum::{
    body::Body,
    http::{Request, StatusCode},
    routing::get,
    Router,
};
use std::net::SocketAddr;
use tower_http::trace::{self, TraceLayer};
use tracing::{error, info_span, instrument, Level, Span};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use extractor::HeaderExtractor;

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

/// Constructs a span at the info level that will wrap the handling of the
/// incoming request.
fn make_span(request: &Request<Body>) -> Span {
    info_span!("request", method = %request.method(), uri = %request.uri(), version = ?request.version(), headers = ?request.headers())
}

/// Trace context propagation. Associates the current span with the OTel trace
/// of the given request, if any and valid.
pub fn accept_trace(request: Request<Body>) -> Request<Body> {
    // Current context, if no or invalid data is received.
    let parent_context = opentelemetry::global::get_text_map_propagator(|propagator| {
        propagator.extract(&HeaderExtractor(request.headers()))
    });
    Span::current().set_parent(parent_context);
    request
}
