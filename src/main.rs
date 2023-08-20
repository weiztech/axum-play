use axum::body::HttpBody;
use axum::handler::Handler;
use axum::response::IntoResponse;
use axum::{
    body::Body,
    extract::Path,
    http::{Request, StatusCode},
    middleware::{self, Next},
    response::Response,
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Pointer};
use tracing::error;
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
    let response = next.run(request).await;
    let status = response.status();
    println!("my middleware end {:?} {:?}", status, response);
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
    tracing_subscriber::fmt::init();

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        // `POST /users` goes to `create_user`
        .route("/users", post(add_update_user))
        .route("/users/:user_id", get(get_user))
        .route("/users/:user_id", put(add_update_user))
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

async fn add_update_user(path: Option<Path<String>>, Json(payload): Json<CreateUser>) -> Response {
    println!("User id {:?} {:?}", path, payload);
    let user_id = match path {
        Some(val) => {
            let is_num = val.0.parse::<u64>();
            if is_num.is_ok() {
                is_num.unwrap()
            } else {
                0
            }
        }
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
