use crate::common::response::ErrorResponse;
use axum::async_trait;
use axum::extract::rejection::JsonRejection;
use axum::extract::{FromRequest, Json, Request};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::de::DeserializeOwned;
use std::fmt::Debug;
use validator::{Validate, ValidationErrors};

pub trait ValidateValue {
    fn validate(&self) -> Result<(), ValidationErrors>;
}

#[async_trait]
impl<T: Debug + Validate> ValidateValue for Json<T> {
    fn validate(&self) -> Result<(), ValidationErrors> {
        self.0.validate()
    }
}

#[derive(Debug)]
pub enum ValidateRejection {
    JsonRejection(JsonRejection),
    ValidationErrors(ValidationErrors),
}

impl ValidateRejection {
    fn to_error_response(self) -> ErrorResponse {
        match self {
            ValidateRejection::ValidationErrors(err) => {
                ErrorResponse::from(err)
            }
            ValidateRejection::JsonRejection(err) => ErrorResponse::from(err),
        }
    }
}

impl From<JsonRejection> for ValidateRejection {
    fn from(value: JsonRejection) -> Self {
        Self::JsonRejection(value)
    }
}

impl From<ValidationErrors> for ValidateRejection {
    fn from(value: ValidationErrors) -> Self {
        Self::ValidationErrors(value)
    }
}

impl IntoResponse for ValidateRejection {
    fn into_response(self) -> Response {
        (StatusCode::BAD_REQUEST, Json(self.to_error_response()))
            .into_response()
    }
}

#[derive(Debug)]
pub struct JSONValidate<T>(pub T);

#[async_trait]
impl<T, S> FromRequest<S> for JSONValidate<T>
where
    S: Send + Sync,
    T: DeserializeOwned + Validate,
{
    type Rejection = ValidateRejection;

    async fn from_request(
        req: Request,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state).await?;
        value.validate()?;
        Ok(Self(value))
    }
}
