use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::HashMap;

use std::fmt::Debug;
use std::string::String;
use validator::{Validate, ValidationErrors, ValidationErrorsKind};

static JSON_REJECTION_MESSAGE: &str = "Invalid json format";

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct PaginationOptions {
    previous: Option<String>,
    #[validate(range(min = 1, max = 25, message = "invalid range value"))]
    limit: Option<u32>,
}

pub struct ListResponse<T> {
    items: Vec<T>,
    pagination: PaginationOptions,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<HashMap<String, Cow<'static, str>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<Cow<'static, str>>,
}

impl ErrorResponse {
    pub fn create_error(error_message: &'static str) -> Self {
        Self {
            errors: None,
            error: Some(Cow::Borrowed(error_message)),
        }
    }
}

impl From<ValidationErrors> for ErrorResponse {
    fn from(value: ValidationErrors) -> Self {
        let errors: HashMap<String, Cow<'static, str>> = value
            .errors()
            .iter()
            .filter_map(|(k, v)| {
                if let ValidationErrorsKind::Field(errors) = v {
                    Some((k.to_string(), errors[0].message.clone().unwrap()))
                } else {
                    None
                }
            })
            .collect();
        Self {
            errors: Some(errors),
            error: None,
        }
    }
}

impl From<JsonRejection> for ErrorResponse {
    fn from(_: JsonRejection) -> Self {
        Self {
            errors: None,
            error: Some(Cow::Borrowed(JSON_REJECTION_MESSAGE)),
        }
    }
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> Response {
        (StatusCode::BAD_REQUEST, Json(self)).into_response()
    }
}
