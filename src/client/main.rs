use reqwest::{
    header::{HeaderName, HeaderValue},
    StatusCode,
};
use std::{collections::HashMap, io::BufRead, net::SocketAddr};
use tokio::join;
use tower::ServiceBuilder;
use tower_http::trace::{self, TraceLayer};
use tracing::{error, info_span, instrument, span_enabled, Level, Span};

use opentelemetry::global::ObjectSafeSpan;
use opentelemetry::propagation::TextMapPropagator;
use opentelemetry::trace::{FutureExt, SpanBuilder};
use opentelemetry::{
    global,
    trace::{SpanKind, TraceContextExt, Tracer},
    Context, KeyValue,
};
use opentelemetry_http::HeaderInjector;
use opentelemetry_sdk::export::trace::SpanExporter;
use opentelemetry_sdk::{propagation::TraceContextPropagator, trace::TracerProvider};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

fn init_tracer() {
    global::set_text_map_propagator(TraceContextPropagator::new());

    // Choose an exporter like `opentelemetry_stdout::SpanExporter`
    // Install stdout exporter pipeline to be able to retrieve the collected spans.
    // For the demonstration, use `Sampler::AlwaysOn` sampler to sample all traces.
    let provider = TracerProvider::builder()
        .with_simple_exporter(opentelemetry_stdout::SpanExporter::default())
        .build();
    // Send traces through this and then they get sent to wherever they need to
    // go - DD or stdout.

    // Tracer: input (feed it spans)
    // Exporter: output for traces, not logs (formats and outputs)

    // Logger would be totally separate.
    // It just prints to stdout and in AWS those get collected and forwarded to DD.

    global::set_tracer_provider(provider);
}

#[tokio::main]
async fn main() {
    init_tracer();
    // tracing_subscriber::registry()
    //     .with(EnvFilter::from_default_env())
    //     .with(fmt::layer().pretty())
    //     .init();

    let stdin = std::io::stdin();

    let client = HelloClient::new();

    for line in stdin.lock().lines() {
        handle_hello(&client, &line.unwrap()).await;
    }
}

pub fn client_tracer_name() -> String {
    "client".to_owned()
}

pub async fn handle_hello(client: &HelloClient, word: &str) {
    let tracer = global::tracer(client_tracer_name());

    let ctx = Context::default();
    let span = tracer
        .span_builder(String::from("handle_hello"))
        .with_kind(SpanKind::Server)
        .start(&tracer);
    let ctx = ctx.with_span(span);
    // Do impressive work.
    client.hello(&ctx, word).await;
    // Should I end the span here?
    ctx.span().end()
}

pub struct HelloClient {
    http_client: reqwest::Client,
}

impl HelloClient {
    pub fn new() -> Self {
        Self {
            http_client: reqwest::Client::default(),
        }
    }

    pub async fn hello(&self, ctx1: &Context, word: &str) {
        let span_name = "call_hello";
        let tracer = global::tracer(client_tracer_name());
        let span = SpanBuilder::from_name(span_name)
            .with_kind(SpanKind::Client)
            .start_with_context(&tracer, ctx1);
        let ctx2 = ctx1.with_span(span);

        // How to log???

        let mut req = self
            .http_client
            .get(format!("http://127.0.0.1:3000/{word}"))
            .build()
            .unwrap();
        global::get_text_map_propagator(|propagator| {
            propagator.inject_context(&ctx2, &mut HeaderInjector(req.headers_mut()))
        });
        let res = self.http_client.execute(req).await.unwrap();
        println!("response status code: {:?}", res.status());
        println!("response: {:?}", res);

        // let propagator = TraceContextPropagator::new();
        // let mut fields = HashMap::new();
        // propagator.inject_context(&context, &mut fields);
        // let headers = fields
        //     .into_iter()
        //     .map(|(k, v)| {
        //         (
        //             HeaderName::try_from(k).unwrap(),
        //             HeaderValue::try_from(v).unwrap(),
        //         )
        //     })
        //     .collect();

        // println!("HEADERS: {:?}", headers);

        // let resp = client
        //     .get("http://127.0.0.1:3000/")
        //     .headers(headers)
        //     .send()
        //     .await
        //     .unwrap();
        // println!("response status code: {:?}", resp.status());
    }
}
