use axum::{debug_handler, response::IntoResponse, Json};

use crate::common::error::Result;
use crate::common::extractor::JSONValidate;
use crate::db::extractors::{ConnectionPool, DatabaseConnection};
use crate::users::schema::{RegisterEmail, UserPasswordLogin};
use crate::users::{db::create_user, models::User};

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
    let user: User = create_user(
        conn,
        &payload.email,
        Some(payload.password.as_str()),
        payload.first_name.as_deref(),
        payload.last_name.as_deref(),
    )
    .await?;
    Ok(Json(user).into_response())
}
