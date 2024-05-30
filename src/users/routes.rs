use crate::users::views::{password_login, user_list, user_register};
use crate::ConnectionPool;
use axum::routing::{delete, get, post, Router};

pub fn auth_routes() -> Router<ConnectionPool> {
    Router::new()
        .route("/auth/password", post(password_login))
        .route("/auth/register", post(user_register))
        .route("/list", get(user_list))
}

/*
pub fn auth_routes() -> Router<ConnectionPool>{
    Router::new()
        .route("/register", post(user_register))
}*/
