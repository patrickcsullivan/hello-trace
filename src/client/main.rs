use reqwest::StatusCode;
use std::{io::BufRead, net::SocketAddr};
use tower::ServiceBuilder;
use tower_http::trace::{self, TraceLayer};
use tracing::{error, info_span, instrument, Level, Span};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() {
    let stdin = std::io::stdin();
    for _line in stdin.lock().lines() {
        send().await;
    }
}

async fn send() {
    let client = reqwest::Client::new();
    let url = "http://localhost:3000/";
    let resp = client.get(url).send().await.unwrap();
    println!("response status code: {:?}", resp.status());
}
