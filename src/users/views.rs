use axum::{debug_handler, response::IntoResponse, Json};

use crate::common::error::Result;
use crate::common::extractor::JSONValidate;
use crate::db::extractors::{ConnectionPool, DatabaseConnection};
use crate::users::db::create_user;
use crate::users::schema::{RegisterEmail, UserPasswordLogin};

#[debug_handler]
pub async fn password_login(
    Json(payload): Json<UserPasswordLogin>,
) -> impl IntoResponse {
    println!("Payload {:?}", payload);
    "OK"
}

#[debug_handler(state=ConnectionPool)]
pub async fn user_register(
    DatabaseConnection(conn): DatabaseConnection,
    JSONValidate(payload): JSONValidate<RegisterEmail>,
) -> Result<impl IntoResponse> {
    let user = create_user(conn, payload).await?;
    Ok(Json(user).into_response())
}
