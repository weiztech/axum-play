use chrono::{DateTime, Utc};
use core::fmt::Debug;

use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::common::response::PaginationOptions;
use crate::common::to_sql::ToSqlString;
use crate::common::utils::EMAIL_SUFFIX;

#[derive(Deserialize, Debug)]
pub struct UserProfile {
    first_name: Option<String>,
    last_name: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UserPasswordLogin {
    #[validate(
        email(message = "invalid email value"),
        length(min = 5, max = 60, message = "invalid field length"),
        regex(path = "EMAIL_SUFFIX", message = "invalid email format")
    )]
    email: String,
    password: String,
}

#[derive(Debug, Validate, Deserialize, Serialize)]
pub struct RegisterEmail {
    #[validate(length(max = 50, message = "invalid field length"))]
    pub first_name: Option<String>,
    #[validate(length(max = 50, message = "invalid field length"))]
    pub last_name: Option<String>,
    #[validate(
        email(message = "invalid email value"),
        length(min = 5, max = 60, message = "invalid field length"),
        regex(path = "EMAIL_SUFFIX", message = "invalid email format"),
        required(message = "field is required")
    )]
    pub email: Option<String>,
    #[validate(
        length(min = 5, max = 100, message = "invalid field length"),
        must_match(
            other = "new_password",
            message = "not match with new password"
        ),
        required(message = "field is required")
    )]
    pub password: Option<String>,
    #[validate(
        length(min = 5, max = 100, message = "invalid field length"),
        required(message = "field is required")
    )]
    new_password: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct EmailChange {
    email: String,
    password: String,
    new_password: String,
}

#[derive(Debug, Deserialize, Validate, Serialize, ToSqlString)]
pub struct UserQuery {
    #[validate(length(min = 3, max = 100, message = "invalid field length"))]
    pub email: Option<String>,
    pub is_active: Option<bool>,
    #[validate(length(min = 3, max = 50, message = "invalid field length"))]
    pub first_name: Option<String>,
    #[validate(length(min = 3, max = 50, message = "invalid field length"))]
    pub last_name: Option<String>,
    pub create_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ProfileChange {
    #[serde(default, with = "::serde_with::rust::double_option")]
    pub first_name: Option<Option<String>>,
    #[serde(default, with = "::serde_with::rust::double_option")]
    #[validate(length(min = 3, max = 50, message = "invalid field length"))]
    pub last_name: Option<Option<String>>,
}
