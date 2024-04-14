use axum::{debug_handler, response::IntoResponse, Json};
use chrono::Utc;
use tokio_postgres::types::ToSql;

use crate::common::error::AppError;
use crate::common::extractor::JSONValidate;
use crate::db::extractors::{ConnectionPool, DatabaseConnection};
use crate::users::db::create_user;
use crate::users::models::{ToUser, User};
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
) -> Result<impl IntoResponse, AppError> {
    let user = create_user(conn, payload).await?;
    Ok(Json(user).into_response())
}
