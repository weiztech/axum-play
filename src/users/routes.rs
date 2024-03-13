use axum::routing::{Router, get, post, delete};
use crate::ConnectionPool;
use crate::users::views::{
    password_login,
    user_register
};


pub fn login_routes() -> Router<ConnectionPool>{
    Router::new()
        .route("/password", post(password_login))
        // .route("/register", post(user_register))
}

/*
pub fn auth_routes() -> Router<ConnectionPool>{
    Router::new()
        .route("/register", post(user_register))
}*/