use axum::routing::{Router, get, post, delete};
use crate::users::views::{
    password_login
};


pub fn login_routes() -> Router{
    Router::new()
        .route("/password", post(password_login))
}