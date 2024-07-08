use crate::common::response::ErrorResponse;
use axum::async_trait;
use axum::extract::rejection::{JsonRejection, QueryRejection};
use axum::extract::{FromRequest, FromRequestParts, Json, Query, Request};
use axum::http::request::Parts;
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
    QueryRejection(QueryRejection),
    ValidationErrors(ValidationErrors),
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

impl From<QueryRejection> for ValidateRejection {
    fn from(value: QueryRejection) -> Self {
        Self::QueryRejection(value)
    }
}

impl IntoResponse for ValidateRejection {
    fn into_response(self) -> Response {
        println!("ValidateRejection: {:?}", self);
        let error_response = match self {
            ValidateRejection::JsonRejection(err) => ErrorResponse::from(err),
            ValidateRejection::QueryRejection(err) => {
                ErrorResponse::create_error("Invalid query params format")
            }
            ValidateRejection::ValidationErrors(err) => {
                ErrorResponse::from(err)
            }
        };
        error_response.into_response()
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

#[derive(Debug)]
pub struct QueryValidate<T>(pub T);

#[async_trait]
impl<T, S> FromRequestParts<S> for QueryValidate<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = ValidateRejection;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let Query(query) = Query::<T>::from_request_parts(parts, state).await?;
        query.validate()?;
        Ok(Self(query))
    }
}
