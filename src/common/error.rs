use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use std::fmt::Debug;
use tracing::error;

pub enum AppError {
    UnexpectedError,
    FatalError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let body = match self {
            AppError::UnexpectedError => "something went wrong".to_string(),
            AppError::FatalError(string) => string,
        };

        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}

pub struct InvalidPayload<T>(pub T);

impl<T> IntoResponse for InvalidPayload<T>
where
    T: Debug,
{
    fn into_response(self) -> Response {
        error!("\nPAYLOAD ERROR: {:?}", self.0);
        (StatusCode::BAD_REQUEST, "Invalid Payload").into_response()
    }
}


pub fn internal_error<E>(err: E) -> AppError
    where
        E: std::error::Error,
{
    AppError::UnexpectedError
}