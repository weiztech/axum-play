use axum::extract::rejection::JsonRejection;
use serde::Serialize;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::Debug;
use validator::{ValidationErrors, ValidationErrorsKind};

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub errors: HashMap<&'static str, Cow<'static, str>>,
}

impl From<ValidationErrors> for ErrorResponse {
    fn from(value: ValidationErrors) -> Self {
        let errors: HashMap<&'static str, Cow<'static, str>> = value
            .errors()
            .iter()
            .filter_map(|(k, v)| {
                if let ValidationErrorsKind::Field(errors) = v {
                    Some((*k, errors[0].message.clone().unwrap()))
                } else {
                    None
                }
            })
            .collect();
        Self { errors }
    }
}

impl From<JsonRejection> for ErrorResponse {
    fn from(value: JsonRejection) -> Self {
        Self {
            errors: HashMap::from([("message", Cow::Owned(value.body_text()))]),
        }
    }
}
