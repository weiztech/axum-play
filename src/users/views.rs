use axum::{debug_handler, response::IntoResponse, Json};
use chrono::Utc;
use tokio_postgres::types::ToSql;

use crate::common::error::AppError;
use crate::common::extractor::JSONValidate;
use crate::db::extractors::DatabaseConnection;
use crate::users::models::User;
use crate::users::schema::{UserPasswordLogin, UserRegisterPassword};

#[debug_handler]
pub async fn password_login(
    Json(payload): Json<UserPasswordLogin>,
) -> impl IntoResponse {
    println!("Payload {:?}", payload);
    "OK"
}

/*#[debug_handler(state=UserRegisterPassword)]*/
pub async fn user_register(
    DatabaseConnection(conn): DatabaseConnection,
    JSONValidate(payload): JSONValidate<UserRegisterPassword>,
) -> Result<impl IntoResponse, AppError> {
    let now = Utc::now();
    let username =
        payload.first_name.to_string() + now.timestamp().to_string().as_str();
    let query = "INSERT INTO users (\
    email, username, first_name, last_name, password, create_at) \
    VALUES ($1, $2, $3, $4, $5, $6) RETURNING id, create_at";
    let values: &[&(dyn ToSql + Sync)] = &[
        &payload.email,
        &username,
        &payload.first_name,
        &payload.last_name,
        &payload.password,
        &now,
    ];
    let user_row = conn.query_one(query, values).await?;
    let user = User {
        id: user_row.get(0),
        email: payload.email,
        image: None,
        username,
        first_name: payload.first_name,
        last_name: payload.last_name,
        create_at: user_row.get(1),
        update_at: None,
        last_login: None,
    };
    Ok(Json(user).into_response())
}
