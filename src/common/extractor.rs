use axum::async_trait;
use axum::extract::rejection::JsonRejection;
use axum::extract::{FromRequest, Json, Request};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tracing_subscriber::fmt::format::json;
use validator::{Validate, ValidationErrors};

pub trait ValidateValue {
    fn validate(&self) -> Result<(), ValidationErrors>;
}

#[async_trait]
impl<T: Debug + Validate> ValidateValue for Json<T> {
    fn validate(&self) -> Result<(), ValidationErrors> {
        println!("Validate {:?}", self.0.validate());
        println!("ExtractValue {:?}", self);
        self.0.validate()
    }
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse{
    message: String
}


#[derive(Debug)]
pub enum ValidateRejection {
    JsonRejection(JsonRejection),
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

impl IntoResponse for ValidateRejection {
    fn into_response(self) -> Response {
        // println!("ValidateRejection {:?}", self);
        match self {
            ValidateRejection::ValidationErrors(err) => {
                let message = err.to_string().replace('\n', ", ");
                (
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse{ message })
                ).into_response()
            }
            ValidateRejection::JsonRejection(err) => (
                (
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse{ message: err.body_text() })
                ).into_response()
            ),
        }
    }
}

#[derive(Debug)]
pub struct JSONValidate<T>(pub T);


#[async_trait]
impl<T, S> FromRequest<S> for JSONValidate<T>
where
    S: Send + Sync,
    T: DeserializeOwned + Validate,
    // Json<T>: FromRequest<S, Rejection = ValidateRejection>,
    // ValidateRejection: From<<T as FromRequest<S>>::Rejection>,
{
    type Rejection = ValidateRejection;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state).await?;
        // value.validate().expect("TODO: panic message");
        // println!("JSONValidatet HERE {:?}", value.validate());
        value.validate()?;
        Ok(Self(value))
    }
}

/*
impl<T> IntoResponse for JSONValidate<T>
where
    T: IntoResponse,
{
    fn into_response(self) -> Response {
        self.0.into_response()
    }
}*/
