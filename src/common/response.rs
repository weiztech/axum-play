use axum::extract::rejection::JsonRejection;
use serde::Serialize;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::Debug;
use validator::{ValidationErrors, ValidationErrorsKind};

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<HashMap<String, Cow<'static, str>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
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
    fn from(value: JsonRejection) -> Self {
        Self {
            errors: None,
            error: Some(value.body_text()),
        }
    }
}
