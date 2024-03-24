use crate::common::extractor::ValidateRejection;
use crate::common::response::ErrorResponse;
use axum::{
    extract::Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::Debug;
use std::ptr::null;
use tokio_postgres::error::{DbError, SqlState};
use tracing::error;
use validator::ValidationErrors;

use once_cell::sync::Lazy;
use regex::Regex;

static DB_FIELD_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"Key \((.*?)\)=").expect("Invalid regex pattern"));

pub enum AppError {
    UnexpectedError,
    FatalError(String),
    DBError(tokio_postgres::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let body = match self {
            AppError::UnexpectedError => "something went wrong".to_string(),
            AppError::FatalError(string) => string,
            AppError::DBError(error) => {
                return match error.as_db_error() {
                    Some(err) if err.code().code() == "23505" => {
                        let field_name =
                            if let Some(caps) = DB_FIELD_REGEX.captures(err.detail().unwrap()) {
                                caps.get(1).map_or("message", |m| m.as_str())
                            } else {
                                "message"
                            }
                            .to_string();

                        let errors =
                            HashMap::from([(field_name, Cow::Owned("Already exists".to_string()))]);
                        Json(ErrorResponse {
                            error: None,
                            errors: Some(errors),
                        })
                        .into_response()
                    }
                    Some(err) => {
                        error!("Unexpected - DB save error code {:?}", err);
                        "Unexpected save error".into_response()
                    }
                    _ => {
                        error!("Unexpected - DB save error {:?}", error);
                        "Unexpected Error".into_response()
                    }
                }
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

pub fn internal_error<E>(_err: E) -> AppError
where
    E: std::error::Error,
{
    AppError::UnexpectedError
}
