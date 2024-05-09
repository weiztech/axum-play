use chrono::Utc;
use once_cell::sync::Lazy;
use std::borrow::Cow;
use std::env;
use std::fmt::Debug;
use std::string::ToString;
use uuid::Uuid;
use validator::Validate;

use crate::common::error::{AppError, Result};
use crate::common::utils::{
    uuid7_b62,
    Password::{generate_password_hash, is_valid},
};
use crate::db::extractors::ConnectionPooled;
use crate::users::models::{ToUser, User};

pub static PASSWORD_ITERATION: Lazy<u32> = Lazy::new(|| {
    env::var("PASSWORD_ITERATION")
        .unwrap_or_else(|_| "10000".to_string())
        .parse::<u32>()
        .unwrap()
});

pub async fn create_user<'a>(
    con: ConnectionPooled,
    email: &'a str,
    password: Option<&'a str>,
    first_name: Option<&'a str>,
    last_name: Option<&'a str>,
) -> Result<User<'a>> {
    let now = Utc::now();
    let user_id = uuid7_b62();

    let email_prefix = email.split("@").next().unwrap();
    let user_first_name = match first_name {
        Some(first_name) => first_name,
        _ => email_prefix,
    };

    let username = first_name.unwrap_or_else(|| email_prefix).to_lowercase()
        + &user_id[user_id.len() - 5..];

    let user_password_hash = match password {
        Some(password) => generate_password_hash(
            password,
            user_id.as_str(),
            *PASSWORD_ITERATION,
        )
        .ok_or_else(|| {
            AppError::FatalError("Failed to create user".to_string())
        })?,
        _ => "".to_string(),
    };

    let user = User {
        id: Some(Cow::Owned(user_id)),
        email: Some(email),
        image: None,
        username: Some(Cow::Owned(username)),
        first_name: Some(user_first_name),
        last_name,
        create_at: Some(now),
        update_at: None,
        last_login: None,
    };
    user.validate()?;

    let query = "INSERT INTO users (\
    id, email, username, first_name, last_name, password) \
    VALUES ($1, $2, $3, $4, $5, $6) RETURNING id, create_at";
    con.query_one(
        query,
        &[
            user.id.as_ref().unwrap(),
            &email,
            user.username.as_ref().unwrap(),
            &user_first_name,
            &last_name,
            &user_password_hash,
        ],
    )
    .await?;

    Ok(user)
}
