use core::fmt::Debug;

use serde::Deserialize;
use validator::Validate;

use once_cell::sync::Lazy;
use regex::Regex;

static EMAIL_SUFFIX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\.[a-zA-Z]{2,}$").unwrap());

#[derive(Deserialize, Debug)]
pub(crate) struct UserProfile {
    first_name: Option<String>,
    last_name: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub(crate) struct UserPasswordLogin {
    email: String,
    password: String,
}

#[derive(Debug, Validate, Deserialize)]
pub(crate) struct UserRegisterPassword {
    #[validate(length(min = 3, max = 254, message = "invalid field length"))]
    pub first_name: String,
    #[validate(length(min = 3, max = 254, message = "invalid field length"))]
    pub last_name: Option<String>,
    #[validate(
        email(message = "invalid email"),
        length(min = 5, max = 254, message = "invalid field length"),
        regex(path = "EMAIL_SUFFIX", message = "invalid email format")
    )]
    pub email: String,
    #[validate(
        length(min = 5, max = 254, message = "invalid field length"),
        must_match(
            other = "new_password",
            message = "not match with new password"
        )
    )]
    pub password: String,
    #[validate(length(min = 5, max = 254, message = "invalid field length"))]
    new_password: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct EmailChange {
    email: String,
    password: String,
    new_password: String,
}
