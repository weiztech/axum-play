use core::fmt::Debug;

use serde::Deserialize;
use validator::Validate;

use crate::users::models::{ToUser, User};

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
pub(crate) struct RegisterEmail {
    pub first_name: String,
    pub last_name: Option<String>,
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

impl ToUser for RegisterEmail {
    fn to_user(self) -> Result<User, String> {
        Ok(User {
            id: None,
            email: self.email,
            image: None,
            first_name: Some(self.first_name),
            last_name: self.last_name,
            password: Some(self.password),
            update_at: None,
            last_login: None,
            create_at: None,
            username: None,
        })
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct EmailChange {
    email: String,
    password: String,
    new_password: String,
}
