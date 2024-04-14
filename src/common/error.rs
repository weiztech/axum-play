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

pub enum AppError {
    UnexpectedError,
    FatalError(String),
    DBError(tokio_postgres::Error),
    ValidationErrors(ValidationErrors),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        return match self {
            AppError::UnexpectedError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "something went wrong".to_string(),
            )
                .into_response(),
            AppError::FatalError(string) => {
                (StatusCode::INTERNAL_SERVER_ERROR, string).into_response()
            }
            AppError::DBError(error) => {
                return match error.as_db_error() {
                    Some(err) if err.code().code() == "23505" => {
                        let constraint = err.constraint().unwrap();
                        let first_idx = constraint.find("_").unwrap();
                        let last_idx = constraint.rfind("_").unwrap();
                        let fields: Vec<&str> = constraint
                            [first_idx + 1..last_idx]
                            .split("$$$")
                            .collect();
                        let count_fields = fields.len();
                        let message = match count_fields {
                            1 => "already exists".to_string(),
                            2 => format!(
                                "{} with {} already exists",
                                &fields[0], &fields[1]
                            ),
                            _ => {
                                let comma_separated =
                                    &fields[1..count_fields - 1].join(", ");
                                format!(
                                    "{} with {} and {} already exists",
                                    &fields[0],
                                    comma_separated,
                                    &fields[count_fields - 1]
                                )
                            }
                        }
                        .replace("_", " ");
                        let field_name = fields[0].to_string();
                        // println!("Field name {:?} {} {}", err, field_name, message);
                        let errors =
                            HashMap::from([(field_name, Cow::Owned(message))]);
                        (
                            StatusCode::BAD_REQUEST,
                            Json(ErrorResponse {
                                error: None,
                                errors: Some(errors),
                            }),
                        )
                            .into_response()
                    }
                    Some(err) => {
                        error!("Unexpected - DB save error code {:?}", err);
                        (StatusCode::BAD_REQUEST, "data save error")
                            .into_response()
                    }
                    _ => {
                        error!("Unexpected - DB save error {:?}", error);
                        (StatusCode::BAD_REQUEST, "Failed save").into_response()
                    }
                };
            }
            AppError::ValidationErrors(err) => {
                ErrorResponse::from(err).into_response()
            }
        };
    }
}

impl From<tokio_postgres::Error> for AppError {
    fn from(value: tokio_postgres::Error) -> Self {
        Self::DBError(value)
    }
}

impl From<ValidationErrors> for AppError {
    fn from(value: ValidationErrors) -> Self {
        Self::ValidationErrors(value)
    }
}

pub fn internal_error<E>(_err: E) -> AppError
where
    E: std::error::Error,
{
    AppError::UnexpectedError
}
