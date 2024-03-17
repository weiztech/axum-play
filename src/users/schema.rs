use core::fmt::Debug;

use serde::Deserialize;
use validator::Validate;

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
    #[validate(length(min=3, max=254))]
    pub first_name: String,
    #[validate(length(min=3, max=254))]
    pub last_name: Option<String>,
    #[validate(email, length(min=5, max=254))]
    pub email: String,
    #[validate(length(min=5, max=254), must_match="new_password")]
    pub password: String,
    #[validate(length(min=5, max=254))]
    new_password: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct EmailChange {
    email: String,
    password: String,
    new_password: String,
}

