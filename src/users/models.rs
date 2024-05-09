use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use validator::Validate;

use crate::common::utils::{
    Password::generate_password_hash, EMAIL_SUFFIX, PASSWORD_ITERATION,
};
use uuid::Uuid;

#[derive(Serialize, Debug, Validate)]
pub struct User<'a> {
    pub id: Option<Cow<'a, str>>,
    #[validate(
        email(message = "invalid email value"),
        length(min = 5, max = 254, message = "invalid field length"),
        regex(path = "EMAIL_SUFFIX", message = "invalid email format")
    )]
    pub email: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<&'a str>,
    // #[serde(skip_serializing)]
    // pub password: Option<&'a str>,
    // #[serde(skip_serializing)]
    // pub is_active: Option<&'a bool>,
    // username value should be generated
    pub username: Option<Cow<'a, str>>,
    #[validate(length(min = 1, max = 254, message = "invalid field length"))]
    pub first_name: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(length(min = 1, max = 254, message = "invalid field length"))]
    pub last_name: Option<&'a str>,
    pub create_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_login: Option<DateTime<Utc>>,
}

impl User<'_> {
    pub fn get_password_hash(password: &str, salt_str: &str) -> String {
        generate_password_hash(password, salt_str, *PASSWORD_ITERATION).unwrap()
    }
}

pub trait ToUser {
    fn to_user(self) -> Result<User<'static>, String>;
}
