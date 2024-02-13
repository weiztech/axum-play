use crate::users::models::UserPasswordLogin;
use axum::{
    response::IntoResponse,
    extract::Json,
    debug_handler
};


#[debug_handler]
pub async fn password_login(Json(payload): Json<UserPasswordLogin>) -> impl IntoResponse {
    println!("Payload {:?}", payload);
    "OK"
}
