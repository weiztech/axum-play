use std::fmt::Debug;
use axum::async_trait;
use axum::body::Body;
use axum::extract::rejection::JsonRejection;
use axum::extract::{FromRequest, Request, Json};
use axum::response::{IntoResponse, Response};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use validator::{Validate, ValidationErrors};


#[derive(Debug)]
pub struct ValidateValue<'a, T>(&'a T);

impl<'a, T: Debug> ValidateValue<'a, T> {
    fn is_valid(&self) {
        println!("Validate Value {:?}", self);
    }
}


pub trait ExtractValue{
    fn get_validation(&self);
}


impl<T: Debug> ExtractValue for Json<T>{
    fn get_validation(&self) {
        let value = ValidateValue(&self.0).is_valid();
        println!("ExtractValue {:?}", self);
    }
}

pub struct JSONValidate<T>(pub T);

#[async_trait]
impl<T, S> FromRequest<S> for JSONValidate<T>
where
    S: Send + Sync,
    T: FromRequest<S> + Debug + ExtractValue,
    JsonRejection: From<<T as FromRequest<S>>::Rejection>,
{
    type Rejection = JsonRejection;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let value = T::from_request(req, state).await?;
        // value.validate().expect("TODO: panic message");
        println!("JSONValidatet HERE {:?}", value.get_validation());
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
