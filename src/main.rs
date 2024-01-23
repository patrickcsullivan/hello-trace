use axum::{
    body::{Body, Bytes},
    extract::{Path, State},
    http::{header, HeaderValue},
    http::{Request, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tower::ServiceBuilder;
use tower_http::{
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
    LatencyUnit,
};
use tracing::{debug, error, info, info_span, instrument, span, Level, Span};
use tracing_subscriber::{
    filter::EnvFilter,
    fmt::{
        self,
        format::{self, FmtSpan},
        SubscriberBuilder,
    },
    layer::SubscriberExt,
    util::SubscriberInitExt,
    FmtSubscriber,
};

#[tokio::main]
async fn main() {
    // OTel (Open Telemtry)

    // Subscriber:
    // - Default location for messages to be sent.
    // - Has a Text Map Propogator
    // - Sends to Collector

    // Collector:
    // - The DataDog Agent.

    // Text Map Propogator:
    // - Defines the format because it defines what information we'll accept and
    //   carry through the trace.
    // - Defines what traces are brought between calling functions.

    // - trace parent headers: trace, span
    // - in the collector

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(fmt::layer().compact())
        .init();

    let config = Config {
        addr: "0.0.0.0:3000".to_string(),
    };

    debug!(?config, "starting");

    // let middleware = ServiceBuilder::new().layer(
    //     TraceLayer::new_for_http()
    //         .on_body_chunk(|chunk: &Bytes, latency: std::time::Duration, _: &tracing::Span| {
    //             tracing::trace!(size_bytes = chunk.len(), latency = ?latency, "sending body chunk")
    //         })
    //         .make_span_with(DefaultMakeSpan::new().include_headers(true))
    //         .on_response(DefaultOnResponse::new().include_headers(true).latency_unit(LatencyUnit::Micros)),
    // );

    //let middleware = TraceLayer::new_for_http().make_span_with(make_span);
    let middleware = TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
        .on_response(DefaultOnResponse::new().level(Level::INFO));

    // build our application with a route
    let app = Router::new()
        .layer(middleware)
        // `GET /` goes to `root`
        .route("/", get(hello_world))
        // `POST /users` goes to `create_user`
        .route("/users", post(create_user));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind(config.addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// basic handler that responds with a static string
#[instrument]
async fn hello_world() -> &'static str {
    info!("saying hello world");

    span!(Level::INFO, "my_span");
    let span = info_span!("my span");
    span.in_scope(|| {
        span!(Level::DEBUG, "inside my_span");
    });

    let _ = fancy_business(10).await;
    "Hello, World!"
}

#[instrument]
async fn fancy_business(x: usize) -> usize {
    error!("uh oh!");
    x
}

#[instrument]
async fn create_user(
    // this argument tells axum to parse the request body
    // as JSON into a `CreateUser` type
    Json(payload): Json<CreateUser>,
) -> (StatusCode, Json<User>) {
    // insert your application logic here
    let user = User {
        id: 1337,
        username: payload.username,
    };

    // this will be converted into a JSON response
    // with a status code of `201 Created`
    (StatusCode::CREATED, Json(user))
}

// the input to our `create_user` handler
#[derive(Debug, Deserialize)]
struct CreateUser {
    username: String,
}

// the output to our `create_user` handler
#[derive(Debug, Serialize)]
struct User {
    id: u64,
    username: String,
}

#[derive(Debug)]
struct Config {
    pub addr: String,
}

fn make_span(request: &Request<Body>) -> Span {
    let headers = request.headers();
    info_span!("incoming request", ?headers)
}
