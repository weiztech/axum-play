use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::common::error::AppError;
use crate::common::utils::EMAIL_SUFFIX;

#[derive(Serialize, Debug, Validate)]
pub struct User {
    pub id: Option<String>,
    #[validate(
        email(message = "invalid email value"),
        length(min = 5, max = 254, message = "invalid field length"),
        regex(path = "EMAIL_SUFFIX", message = "invalid email format")
    )]
    pub email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    #[serde(skip_serializing)]
    pub password: Option<String>,
    pub username: Option<String>,
    #[validate(length(min = 3, max = 254, message = "invalid field length"))]
    pub first_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(length(min = 1, max = 254, message = "invalid field length"))]
    pub last_name: Option<String>,
    pub create_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_login: Option<DateTime<Utc>>,
}

pub trait ToUser {
    fn to_user(self) -> Result<User, String>;
}
