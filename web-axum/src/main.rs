mod common;
mod db;
mod users;

use common::error::{internal_error, AppError};
use common::extractor::JSONValidate;
use db::extractors::{ConnectionPool, DatabaseConnection};

use axum::body::HttpBody;
use axum::extract::FromRequest;
use axum::response::IntoResponse;
use axum::{
    async_trait,
    body::{Body, Bytes},
    debug_handler,
    error_handling::HandleErrorLayer,
    extract::{
        DefaultBodyLimit, FromRef, FromRequestParts, MatchedPath, Path,
        Request, State,
    },
    http::{request::Parts, HeaderMap, HeaderName, Method, StatusCode, Uri},
    middleware::{self, Next},
    response::Response,
    routing::{delete, get, post, put},
    BoxError, Json, RequestPartsExt, Router,
};
use serde::{Deserialize, Serialize};
use std::env;
use std::fmt::{Display, Pointer};
use std::string::String;
use std::time::Duration;
use tokio::time::sleep;
use tower::ServiceBuilder;
use tower_http::sensitive_headers::SetSensitiveHeadersLayer;
use tower_http::trace::{self};
use tower_http::{
    catch_panic::CatchPanicLayer,
    classify::ServerErrorsFailureClass,
    classify::StatusInRangeAsFailures,
    trace::{DefaultMakeSpan, DefaultOnRequest, TraceLayer},
};
use tracing::error;
use tracing::{info, Level};
use tracing::{info_span, Span};
use tracing_subscriber::fmt::layer;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};
use validator::Validate;

use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use tokio_postgres::NoTls;

// the input to our `create_user` handler
#[derive(Deserialize, Debug, Validate)]
struct CreateUser {
    id: u64,
    #[validate(
        length(min = 5, message = "exceed allowed min"),
        length(max = 10, message = "exceed allowed max")
    )]
    username: String,
    #[validate(
        length(min = 3, message = "exceed allowed min"),
        length(max = 7, message = "exceed allowed max")
    )]
    address: String,
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

    let trace_layer_http =
        TraceLayer::new_for_http().make_span_with(|request: &Request<Body>| {
            tracing::error_span!(
                "\nHTTP Request ",
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

    let manager = PostgresConnectionManager::new_from_stringlike(
        env::var("DATABASE_STRING").unwrap_or(String::from("")),
        NoTls,
    )
    .unwrap();
    let pool = Pool::builder().build(manager).await.unwrap();

    let auth_routes =
        Router::new().nest("/users", users::routes::auth_routes());

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        .nest("/api", auth_routes)
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
                .layer(CatchPanicLayer::new())
                .layer(trace_layer_http),
        )
        .with_state(pool);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::debug!("De debug 12312");
    tracing::info!("De INFO 123");
    tracing::error!("Err logg");
    axum::serve(listener, app).await.unwrap();
}

// basic handler that responds with a static string

#[debug_handler(state=ConnectionPool)]
async fn root(
    DatabaseConnection(conn): DatabaseConnection,
) -> Result<impl IntoResponse, AppError> {
    // let conn = pool.get().await.map_err(internal_error)?;
    let row = conn
        .query_one("select 1 + 1", &[])
        .await
        .map_err(internal_error)?;
    let two: i32 = row.try_get(0).map_err(internal_error)?;
    println!("Value {:?} {}", row, two);
    // panic!("hello world");
    // sleep(Duration::from_secs(2)).await;
    Ok((StatusCode::OK, String::from("Hello World")))
}

#[debug_handler]
async fn add_update_user(
    path: Option<Path<String>>,
    //JSONValidate(payload): JSONValidate<CreateUser>,
    Json(payload): Json<CreateUser>,
) -> impl IntoResponse {
    println!("User id {:?} {:?}", path, payload);
    let user_id = match path {
        Some(val) => val.0.parse::<u64>().unwrap_or(0),
        None => payload.id,
    };
    if user_id == 0 {
        // return StatusCode::BAD_REQUEST.into_response()
        return AppError::UnexpectedError.into_response();
    }

    if payload.id == 1 {
        return AppError::FatalError(format!(
            "{} {:?}",
            "Error Found, Please check", payload
        ))
        .into_response();
        // panic!("Errr");
    } else if payload.id == 2 {
        return AppError::FatalError("invalid id".to_string()).into_response();
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
        Ok(ref person) => {
            println!("Deserialized person: {:?}", person);
            let json_data = serde_json::to_string(&data.unwrap());
            println!("JSON DATA - {:?}", json_data);
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
