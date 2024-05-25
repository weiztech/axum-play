use chrono::{DateTime, Utc};
use serde::{Serialize};
use std::borrow::Cow;

use crate::common::utils::{
    Password::generate_password_hash, PASSWORD_ITERATION,
};

#[derive(Serialize, Debug)]
pub struct User<'a> {
    pub id: Option<Cow<'a, str>>,
    pub email: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<&'a str>,
    // #[serde(skip_serializing)]
    // pub password: Option<&'a str>,
    // #[serde(skip_serializing)]
    // pub is_active: Option<&'a bool>,
    // username value should be generated
    pub username: Option<Cow<'a, str>>,
    pub first_name: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
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
