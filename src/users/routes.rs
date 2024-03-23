use crate::users::views::{password_login, user_register};
use crate::ConnectionPool;
use axum::routing::{delete, get, post, Router};

pub fn auth_routes() -> Router<ConnectionPool> {
    Router::new()
        .route("/password", post(password_login))
        .route("/register", post(user_register))
}

/*
pub fn auth_routes() -> Router<ConnectionPool>{
    Router::new()
        .route("/register", post(user_register))
}*/
