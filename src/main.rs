use std::env;
use axum::body::HttpBody;
use axum::handler::Handler;
use axum::response::IntoResponse;
use axum::{
    body::{Body, Bytes},
    extract::{Path, MatchedPath},
    http::{Request, StatusCode, HeaderMap},
    middleware::{self, Next},
    response::Response,
    routing::{delete, get, post, put},
    Json, Router,
};
use std::time::Duration;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Pointer};
use tower_http::{
    classify::ServerErrorsFailureClass,
    trace::TraceLayer,
    classify::StatusInRangeAsFailures,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tracing::error;
use tracing::{info_span, Span};
use validator::Validate;

// the input to our `create_user` handler
#[derive(Deserialize, Debug)]
struct CreateUser {
    id: u64,
    username: String,
}

// the output to our `create_user` handler
#[derive(Validate, Debug, Serialize, Deserialize)]
struct User {
    id: u64,
    #[validate(length(max = 1))]
    username: String,
    address: String,
}

async fn my_middleware(request: Request<Body>, next: Next) -> Response {
    // do something with `request`...
    println!("my middleware start");
    let request_info = format!("{:?}", &request.body());
    let request_headers = format!("{:?}", &request.headers());
    let response = next.run(request).await;
    let status = response.status();
    println!(
        "my middleware end: \nStatus: {:?}\n Response: {:?} \nHeaders {:?}\nBody: {:?}",
        status, response, request_headers, request_info
    );
    if status == StatusCode::UNPROCESSABLE_ENTITY {
        println!("err middleware {}", status);
        error!("\n\nStatus {}\nRequest: {:?} \n", status, request_info);
    }
    // do something with `response`...
    response
}

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                // axum logs rejections from built-in extractors with the `axum::rejection`
                // target, at `TRACE` level. `axum::rejection=trace` enables showing those events
                "example_tracing_aka_logging=debug,tower_http=debug,axum::rejection=trace".into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();


    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        // `POST /users` goes to `create_user`
        .route("/users", post(add_update_user))
        .route("/users/:user_id", get(get_user))
        .route("/users/:user_id", put(add_update_user))
        .route("/users/:user_id/delete", delete(delete_user))
        .layer(middleware::from_fn(my_middleware))
        .layer(
            TraceLayer::new(
                StatusInRangeAsFailures::new(400..=599).into_make_classifier()
            )
                /*.on_request(|_request: &Request<_>, _span: &Span| {
                    // You can use `_span.record("some_other_field", value)` in one of these
                    // closures to attach a value to the initially empty field in the info_span
                    // created above.
                    tracing::error!("Req DEBUG {:?}", _request);
                    tracing::error!("Req Span {:?}", _span);
                })
                .on_response(|_response: &Response, _latency: Duration, _span: &Span| {
                    // ...
                    tracing::error!("Resp {:?}", _response);
                })
                .on_body_chunk(|_chunk: &Bytes, _latency: Duration, _span: &Span| {
                    tracing::error!("Body {:?}", _chunk);
                    // ...
                })*/
        )
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<_>| {
                    // Log the matched route's path (with placeholders not filled in).
                    // Use request.uri() or OriginalUri if you want the real path.
                    let matched_path = request
                        .extensions()
                        .get::<MatchedPath>()
                        .map(MatchedPath::as_str);

                    info_span!(
                        "http_requestX",
                        method = ?request.method(),
                        headers = ?request.headers(),
                        path = matched_path,
                        presto = "123123",
                        some_other_field = tracing::field::Empty,
                    )
                })
                /*.on_request(|_request: &Request<_>, _span: &Span| {
                    // You can use `_span.record("some_other_field", value)` in one of these
                    // closures to attach a value to the initially empty field in the info_span
                    // created above.
                    tracing::info!("Req {:?}", _request);
                    tracing::info!("Req Span {:?}", _span);
                })*/
                .on_response(|_response: &Response, _latency: Duration, _span: &Span| {
                    // ...
                    tracing::info!("Resp {:?}", _response);
                    tracing::info!("Resp Latency {:?}", _latency);
                    tracing::info!("Resp Span {:?}", _span);
                })
                .on_body_chunk(|_chunk: &Bytes, _latency: Duration, _span: &Span| {
                    tracing::info!("Body {:?}", _chunk);
                    tracing::info!("Body SPAN {:?}", _span);
                    // ...
                })
                .on_eos(
                    |_trailers: Option<&HeaderMap>, _stream_duration: Duration, _span: &Span| {
                        tracing::info!("EOS {:?}", _trailers);
                    },
                )
                .on_failure(
                    |_error: ServerErrorsFailureClass, _latency: Duration, _span: &Span| {
                        // tracing::error!("on Failure ERRRRR {:?}", _error);
                    },
                ),
        );

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::debug!("De debug 12312");
    tracing::info!("De INFO 123");
    tracing::error!("Err logg");
    axum::serve(listener, app).await.unwrap();
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World! 123"
}

async fn add_update_user(path: Option<Path<String>>, Json(payload): Json<CreateUser>) -> Response {
    println!("User id {:?} {:?}", path, payload);
    let user_id = match path {
        Some(val) => val.0.parse::<u64>().unwrap_or(0),
        None => payload.id,
    };
    if user_id == 0 {
        return StatusCode::BAD_REQUEST.into_response();
    }
    let user = User {
        id: user_id,
        username: payload.username,
        address: String::from("Any where"),
    };
    if payload.id == 1 {
        panic!("Errr")
    };
    let data = r#"
    {
        "id": 43,
        "username": "Joel",
        "address": "+123 55555"
    }"#;
    let data = serde_json::from_str::<User>(data);
    match data {
        Ok(person) => {
            println!("Deserialized person: {:?}", person);
        }
        Err(error) => {
            println!("Failed to deserialize data. Error: {}", error);
        }
    }
    // this will be converted into a JSON response
    // with a status code of `201 Created`
    Json(user).into_response()
}

async fn get_user(Path(user_id): Path<String>) -> (StatusCode, Json<User>) {
    let user = User {
        id: user_id.parse::<u64>().unwrap(),
        username: String::from("hello ") + &user_id,
        address: String::from("hello world"),
    };
    (StatusCode::OK, Json(user))
}

async fn delete_user(Path(user_id): Path<String>) -> StatusCode {
    let user_id = user_id;
    if user_id != "1" {
        return StatusCode::NO_CONTENT;
    }
    StatusCode::OK
}
