use std::fmt::{Display, Pointer};
use axum::{
    routing::{get, post, delete},
    extract::{Path, Request},
    http::StatusCode,
    Json, Router,
    response::Response,
    middleware::{self, Next},
    body::Body,
};
use axum::body::HttpBody;
use axum::handler::Handler;
use serde::{Deserialize, Serialize};
use validator::Validate;
use tracing::error;


// the input to our `create_user` handler
#[derive(Deserialize)]
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

async fn my_middleware(
    request: Request<Body>,
    next: Next,
) -> Response {
    // do something with `request`...
    println!("my middleware start");
    let request_info = format!("{:?}", &request.body());
    let response = next.run(request).await;
    let status = response.status();
    println!("my middleware end {:?} {:?}", status, response);
    if status == StatusCode::UNPROCESSABLE_ENTITY{
        println!("err middleware {}", status);
        error!(
            "\n\nStatus {}\nRequest: {:?} \n",
            status, request_info
        );
    }
    // do something with `response`...
    response
}

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        // `POST /users` goes to `create_user`
        .route("/users", post(create_user))
        .route("/users/:user_id", get(get_user))
        .route("/users/:user_id/delete", delete(delete_user))
        .layer(middleware::from_fn(my_middleware));
        // .layer(TraceLayer::new_for_http());

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::debug!("De debug 12312");
    tracing::info!("De INFO");
    tracing::error!("Err log");
    axum::serve(listener, app).await.unwrap();
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World! 123"
}

async fn create_user(
    // this argument tells axum to parse the request body
    // as JSON into a `CreateUser` type
    Json(payload): Json<CreateUser>,
) -> (StatusCode, Json<User>) {
    // insert your application logic here
    let user = User {
        id: payload.id,
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
    (StatusCode::CREATED, Json(user))
}

async fn get_user(Path(user_id): Path<String>) -> (StatusCode, Json<User>) {
    let user = User{
        id: user_id.parse::<u64>().unwrap(),
        username: String::from("hello ") + &user_id,
        address: String::from("hello world"),
    };
    (StatusCode::OK, Json(user))
}

async fn delete_user(Path(user_id): Path<String>) -> StatusCode {
    let user_id = user_id;
    if user_id != "1"{
        return StatusCode::NO_CONTENT
    }
    StatusCode::OK
}