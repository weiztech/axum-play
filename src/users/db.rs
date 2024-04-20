use chrono::Utc;
use once_cell::sync::Lazy;
use std::env;
use std::fmt::Debug;
use std::string::ToString;
use validator::Validate;

use crate::common::error::Result;
use crate::common::utils::{
    uuid7_b62,
    Password::{generate_password_hash, is_valid},
};
use crate::db::extractors::ConnectionPooled;
use crate::users::models::{ToUser, User};

static PASSWORD_ITERATION: Lazy<u32> = Lazy::new(|| {
    env::var("PASSWORD_ITERATION")
        .unwrap_or_else(|_| "10000".to_string())
        .parse::<u32>()
        .unwrap()
});

pub async fn create_user<T>(con: ConnectionPooled, user_data: T) -> Result<User>
where
    T: ToUser + Debug,
{
    let mut user = user_data.to_user().unwrap();
    user.validate()?;

    let now = Utc::now();
    let user_id = uuid7_b62();
    if user.first_name.is_some() {
        user.username = Some(
            user.first_name.as_ref().unwrap().to_string().to_lowercase()
                + now.timestamp().to_string().as_str(),
        );
    }

    if user.password.is_some() {
        let password = user.password.as_ref().unwrap();
        let password_hash = generate_password_hash(
            password,
            user_id.as_str(),
            *PASSWORD_ITERATION,
        );
        if password_hash.is_none() {
            panic!("Failed to generate password {}", password);
        }

        user.password = password_hash;
    };

    let query = "INSERT INTO users (\
    id, email, username, first_name, last_name, password) \
    VALUES ($1, $2, $3, $4, $5, $6) RETURNING id, create_at";
    let user_row = con
        .query_one(
            query,
            &[
                &user_id,
                &user.email,
                &user.username.as_ref().unwrap(),
                &user.first_name.as_ref().unwrap(),
                &user.last_name.as_ref().unwrap(),
                &user.password.as_ref().unwrap(),
            ],
        )
        .await?;

    user.id = Some(user_row.get(0));
    user.create_at = Some(user_row.get(1));
    Ok(user)
}
