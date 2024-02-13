use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
struct User {
    id: u64,
    email: String,
    first_name: String,
    last_name: String,
    slug: String,
}

#[derive(Debug, Deserialize)]
pub struct UserPasswordLogin {
    email: String,
    password: String,
}

#[derive(Debug)]
pub struct UserRegisterPassword {
    email: String,
    password: String,
    new_password: String,
}
