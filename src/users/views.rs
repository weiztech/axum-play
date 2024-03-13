use crate::users::schema::{UserPasswordLogin, UserRegisterPassword};
use axum::{
    response::IntoResponse,
    Json,
    debug_handler
};
use axum_valid::Valid;



#[debug_handler]
pub async fn password_login(Json(payload): Json<UserPasswordLogin>) -> impl IntoResponse {
    println!("Payload {:?}", payload);
    "OK"
}

/*#[debug_handler(state=UserRegisterPassword)]*/
pub async fn user_register(
    payload: Valid<Json<UserRegisterPassword>>
) -> impl IntoResponse {
    println!("Payload {:?}", payload);
    "OK"
}
