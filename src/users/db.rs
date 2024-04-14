use chrono::Utc;
use std::fmt::Debug;
use validator::Validate;

use crate::common::error::AppError;
use crate::db::extractors::ConnectionPooled;
use crate::users::models::{ToUser, User};

pub async fn create_user<T>(
    con: ConnectionPooled,
    user_data: T,
) -> Result<User, AppError>
where
    T: ToUser + Debug,
{
    let mut user = user_data.to_user().unwrap();
    user.validate()?;

    let now = Utc::now();
    if user.first_name.is_some() {
        user.username = Some(
            user.first_name.as_ref().unwrap().to_string()
                + now.timestamp().to_string().as_str(),
        );
    }

    let query = "INSERT INTO users (\
    email, username, first_name, last_name, password) \
    VALUES ($1, $2, $3, $4, $5) RETURNING id, create_at";
    let user_row = con
        .query_one(
            query,
            &[
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
