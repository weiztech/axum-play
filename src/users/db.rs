use chrono::Utc;
use std::fmt::Debug;
use validator::Validate;
use uuid::Uuid;
use base62;

use crate::common::error::{Result};
use crate::db::extractors::ConnectionPooled;
use crate::users::models::{ToUser, User};

pub async fn create_user<T>(
    con: ConnectionPooled,
    user_data: T,
) -> Result<User>
where
    T: ToUser + Debug,
{
    let mut user = user_data.to_user().unwrap();
    user.validate()?;

    let now = Utc::now();
    if user.first_name.is_some() {
        user.username = Some(
            user.first_name.as_ref().unwrap().to_string().to_lowercase()
                + now.timestamp().to_string().as_str(),
        );
    }

    let user_id = base62::encode(Uuid::now_v7().as_u128());
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
