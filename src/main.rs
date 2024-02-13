extern crate core;

mod users;
mod common;

use common::error::{AppError, InvalidPayload};
use common::extractor::{JSONValidate};

use std::string::String;
use axum::body::HttpBody;
use axum::handler::Handler;
use axum::response::IntoResponse;
use axum::{
    debug_handler,
    body::{Body, Bytes},
    error_handling::HandleErrorLayer,
    extract::{DefaultBodyLimit, MatchedPath, Path, Request},
    http::{HeaderMap, HeaderName, Method, StatusCode, Uri},
    middleware::{self, Next},
    response::Response,
    routing::{delete, get, post, put},
    BoxError, Json, RequestPartsExt, Router,
};
use http_body_util::BodyExt;
use http_body_util::Full;
use serde::{Deserialize, Serialize};
use std::env;
use std::fmt::{Display, Pointer};
use std::time::Duration;
use tokio::time::sleep;
use tower::ServiceBuilder;
use tower_http::sensitive_headers::SetSensitiveHeadersLayer;
use tower_http::trace::{self};
use tower_http::{
    classify::ServerErrorsFailureClass,
    classify::StatusInRangeAsFailures,
    trace::{DefaultMakeSpan, TraceLayer, DefaultOnRequest},
};
use tracing::error;
use tracing::{info, Level};
use tracing::{info_span, Span};
use tracing_subscriber::fmt::layer;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};
use validator::Validate;

// the input to our `create_user` handler
#[derive(Deserialize, Debug, Clone)]
struct CreateUser {
    id: u64,
    username: String,
}

// the output to our `create_user` handler
#[derive(Validate, Debug, Serialize, Deserialize)]
struct User<'a> {
    id: u64,
    #[validate(length(max = 1))]
    username: &'a str,
    address: &'a str,
}

async fn my_middleware(request: Request, next: Next) -> Response {
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

async fn new_middleware(request: Request, next: Next) -> Response {
    let (parts, body) = request.into_parts();
    let p_parts = format!("{:?}", parts);

    // this wont work if the body is an long running stream
    let bytes = body
        .collect()
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response())
        .unwrap()
        .to_bytes();

    let response = next
        .run(Request::from_parts(parts, Body::from(bytes.clone())))
        .await;
    let status_code = response.status();
    if status_code != StatusCode::OK && status_code != StatusCode::CREATED {
        error!(
            "\nStatus {}\nRequest: {:?}\nBody: {:?}",
            status_code, p_parts, &bytes
        );
    }
    info!("Response new Middleware {:?}", response);
    response
}

async fn handle_timeout_error(
    // `Method` and `Uri` are extractors so they can be used here
    method: Method,
    uri: Uri,
    // the last argument must be the error itself
    err: BoxError,
) -> (StatusCode, String) {
    let message = format!("`{} {}` failed with {}", method, uri, err);
    error!("TIMEOUT: {message}");
    (StatusCode::INTERNAL_SERVER_ERROR, message)
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

    // let handle_logging = ServiceBuilder::new()
    //     .layer(HandleErrorLayer::new(handle_timeout_error));

    let trace_layer_http = TraceLayer::new_for_http()
        .make_span_with(|request: &Request<Body>| {
            tracing::error_span!("\nHTTP Request ",
                "\nUrl: {:?}\nHeaders: {:?}\n",
                request.uri().path_and_query(),
                request.headers()
            )
        });
        /*
        .on_request(());
        .on_response(
            |response: &Response<Body>, latency: Duration, _span: &Span| {
                println!("SPAN RESP {:?}", _span);
                let message = format!(
                    "\nHTTP {:?} - \nHTTP Response Time: ({:?})",
                    response, latency
                );
                if response.status().is_success() {
                    tracing::info!(message);
                } else {
                    tracing::error!(message)
                }
            },
        )
        .on_eos(
            |trailers: Option<&HeaderMap>, stream_duration: Duration, _span: &Span| {
                tracing::error!("stream closed after {:?}", stream_duration)
            },
        )
        .on_failure(
            |error: ServerErrorsFailureClass, latency: Duration, _span: &Span| {
                tracing::error!("something went wrong")
            },
        );*/

    let api_routes_v1 = Router::new()
        .nest("/login", users::routes::login_routes());

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .nest("/api/v1", api_routes_v1)
        .route("/", get(root))
        // `POST /users` goes to `create_user`
        .route("/users", post(add_update_user))
        .route("/users/:user_id", get(get_user))
        .route("/users/:user_id", put(add_update_user))
        .route("/users/:user_id/delete", delete(delete_user))
        .layer(
            ServiceBuilder::new()
                // BODY LIMIT 100 KB
                .layer(DefaultBodyLimit::max(100000))
                // Should response within max 10 seconds
                .layer(HandleErrorLayer::new(|_: BoxError| async {
                    StatusCode::REQUEST_TIMEOUT
                }))
                .timeout(Duration::from_secs(10))
                .layer(SetSensitiveHeadersLayer::new([
                    HeaderName::from_static("user-agent"),
                    HeaderName::from_static("postman-token"),
                ]))
                .layer(trace_layer_http),
        );

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000"
    ).await.unwrap();
    tracing::debug!("De debug 12312");
    tracing::info!("De INFO 123");
    tracing::error!("Err logg");
    axum::serve(listener, app).await.unwrap();
}

// basic handler that responds with a static string
async fn root() -> impl IntoResponse {
    // sleep(Duration::from_secs(2)).await;
    (StatusCode::OK, "Hello, World! 123")
}

#[debug_handler]
async fn add_update_user(
    path: Option<Path<String>>,
    JSONValidate(Json(payload)): JSONValidate<Json<CreateUser>>) -> impl IntoResponse {
    println!("User id {:?} {:?}", path, payload);
    let user_id = match path {
        Some(val) => val.0.parse::<u64>().unwrap_or(0),
        None => payload.id,
    };
    if user_id == 0 {
        // return StatusCode::BAD_REQUEST.into_response()
        return AppError::UnexpectedError.into_response()
    }

    if payload.id == 1 {
        return AppError::FatalError(
            format!("{} {:?}", "Error Found, Please check", payload),
        ).into_response()
        // panic!("Errr");
    }else if payload.id == 2  {
        return InvalidPayload(payload).into_response()
    }

    let address = format!("Address {}", payload.username);
    let user = User {
        id: user_id,
        username: payload.username.as_str(),
        address: address.as_str(),
    };
    info!("add update user {:?}", user);
    let validate = user.validate();
    println!("Validate {:?}", validate);
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

#[debug_handler]
async fn get_user(Path(user_id): Path<String>) -> Response {
    let username_id = user_id.clone() + " Hhaha";
    let user = User {
        id: user_id.parse::<u64>().unwrap(),
        username: &username_id,
        address: "Hello World",
    };
    Json(user).into_response()
}

async fn delete_user(Path(user_id): Path<String>) -> StatusCode {
    let user_id = user_id;
    if user_id != "1" {
        return StatusCode::NO_CONTENT;
    }
    StatusCode::OK
}
