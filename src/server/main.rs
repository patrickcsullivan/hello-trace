mod extractor;

use axum::{
    body::Body,
    extract::{FromRequest, FromRequestParts, Path, State},
    http::{Request, StatusCode},
    middleware,
    response::Response,
    routing::get,
    Extension, Router,
};
use extractor::HeaderExtractor;
use tracing::error;
use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_http::trace::{self, TraceLayer};

use opentelemetry::propagation::TextMapPropagator;
use opentelemetry::trace::{FutureExt, SpanBuilder};
use opentelemetry::{
    global,
    trace::{SpanKind, TraceContextExt, Tracer},
    Context, KeyValue,
};
use opentelemetry::{global::BoxedSpan, trace::Span};
use opentelemetry_http::HeaderInjector;
use opentelemetry_sdk::export::trace::SpanExporter;
use opentelemetry_sdk::{propagation::TraceContextPropagator, trace::TracerProvider};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use opentelemetry_otlp::WithExportConfig;
use axum_tracing_opentelemetry::middleware::{OtelAxumLayer, OtelInResponseLayer};

#[derive(Debug, Clone)]
pub struct AppState {}

impl AppState {
    pub fn new() -> Self {
        Self {}
    }
}

#[tokio::main]
async fn main() {
    init_tracer();

    let app = Router::new()
        .route("/:word", get(handle_echo))
        //.route_layer(axum::middleware::from_fn(extract_context))
        .layer(OtelInResponseLayer::default())
        //start OpenTelemetry trace on incoming request
        .layer(OtelAxumLayer::default())
        .with_state(AppState::new());

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    tracing::info!("listening on {}", addr);
    axum::serve(listener, app).await.unwrap();
}

fn init_tracer() {
    global::set_text_map_propagator(TraceContextPropagator::new());
    // let provider = TracerProvider::builder()
    // .with_simple_exporter(opentelemetry_stdout::SpanExporter::default())
    // .build();
    // global::set_tracer_provider(provider);
    // let tracer = opentelemetry_otlp::new_pipeline()
    //   .tracing()
    //   .with_exporter(opentelemetry_otlp::new_exporter().tonic())
    //   .install_simple();
    init_tracing_opentelemetry::tracing_subscriber_ext::init_subscribers().unwrap();

}

async fn extract_context(
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> Result<Response, StatusCode> {
    let tracer = global::tracer(server_tracer_name());
    let parent_ctx = extract_context_from_request(&req);
    let span = tracer
        .span_builder(req.uri().to_string())
        .with_kind(SpanKind::Server)
        .start_with_context(&tracer, &parent_ctx);
    let ctx = Context::default().with_span(span);
    let resp = next.run(req).with_context(ctx.clone()).await;
    // Maybe change span type based on response.
    ctx.span().end();
    Ok(resp)
}

// Utility function to extract the context from the incoming request headers
fn extract_context_from_request(req: &Request<Body>) -> Context {
    global::get_text_map_propagator(|propagator| {
        propagator.extract(&HeaderExtractor(req.headers()))
    })
}

fn new_span(name: String, kind: SpanKind) -> BoxedSpan {
    let tracer = global::tracer(server_tracer_name());
    tracer.span_builder(name).with_kind(kind).start(&tracer)
}

// Separate async function for the echo endpoint
async fn handle_echo(State(_state): State<AppState>, Path(word): Path<String>) -> String {
    let span = new_span("handle_echo".to_owned(), SpanKind::Server);
    let ctx = Context::current().with_span(span);
    // log something
    let word = do_work(word).with_context(ctx).await;
    // close span?
    word
}
#[tracing::instrument]
async fn do_work(s: String) -> String {
    let span = new_span("do_work".to_owned(), SpanKind::Server);
    let _ctx = Context::current().with_span(span);
    // log something
    // close span?
    error!("some error");
    s
}

fn server_tracer_name() -> String {
    "server".to_string()
}
