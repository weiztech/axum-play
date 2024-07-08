use crate::users::views::{
    delete_user, edit_user, password_login, user_list, user_register,
};
use crate::ConnectionPool;
use axum::routing::{delete, get, patch, post, Router};

pub fn auth_routes() -> Router<ConnectionPool> {
    Router::new()
        .route("/auth/password", post(password_login))
        .route("/auth/register", post(user_register))
        .route("/list", get(user_list))
        .route("/:user_id/change", patch(edit_user))
        .route("/:user_id/delete", delete(delete_user))
}

/*
pub fn auth_routes() -> Router<ConnectionPool>{
    Router::new()
        .route("/register", post(user_register))
}*/
