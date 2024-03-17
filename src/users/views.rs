use crate::users::schema::{UserPasswordLogin, UserRegisterPassword};
use axum::{
    response::IntoResponse,
    Json,
    debug_handler
};
use tokio_postgres::{types::ToSql};
use chrono::Utc;
use crate::common::error::AppError;
use crate::common::extractor::JSONValidate;
use crate::DatabaseConnection;
use crate::users::models::User;


#[debug_handler]
pub async fn password_login(Json(payload): Json<UserPasswordLogin>) -> impl IntoResponse {
    println!("Payload {:?}", payload);
    "OK"
}

/*#[debug_handler(state=UserRegisterPassword)]*/
pub async fn user_register(
    DatabaseConnection(conn): DatabaseConnection,
    JSONValidate(payload): JSONValidate<UserRegisterPassword>
) -> Result<impl IntoResponse, AppError> {
    let now = Utc::now();
    let slug = payload.first_name.to_string() + now.timestamp().to_string().as_str();
    let query = "INSERT INTO users (email, slug, first_name, last_name, password, create_at) \
    VALUES ($1, $2, $3, $4, $5, $6) RETURNING id";
    let values: &[&(dyn ToSql + Sync)] = &[
        &payload.email, &slug, &payload.first_name,
        &payload.last_name.unwrap_or(String::from("")),
        &payload.password, &now
    ];
    println!("Values {:?}", values);
    conn.execute(query, values).await?;
    Ok("OK")
}
