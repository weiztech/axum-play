use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use std::fmt::Debug;
use tracing::error;
use validator::ValidationErrors;
use crate::common::extractor::ValidateRejection;

pub enum AppError {
    UnexpectedError,
    FatalError(String),
    DBError(tokio_postgres::Error)
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let body = match self {
            AppError::UnexpectedError => "something went wrong".to_string(),
            AppError::FatalError(string) => string,
            AppError::DBError(error) => {
                error!("DBError {}", error);
                String::from("Unexpected Error")
            }
        };

        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}

impl From<tokio_postgres::Error> for AppError {
    fn from(value: tokio_postgres::Error) -> Self {
        Self::DBError(value)
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