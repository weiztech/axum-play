use axum::{debug_handler, response::IntoResponse, Json};
use std::borrow::Cow;

use crate::common::error::Result;
use crate::common::extractor::{JSONValidate, QueryValidate};
use crate::common::response::{ListResponse, PaginationOptions};
use crate::db::extractors::{ConnectionPool, DatabaseConnection};
use crate::db::query::Builder;
use crate::users::schema::{RegisterEmail, UserPasswordLogin, UserQuery};
use crate::users::{db::create_user, models::User};

use crate::common::to_sql::ToSqlString;

#[debug_handler]
pub async fn password_login(
    Json(payload): Json<UserPasswordLogin>,
) -> impl IntoResponse {
    println!("Payload {:?}", payload);
    "OK"
}

#[debug_handler(state=ConnectionPool)]
pub async fn user_register(
    DatabaseConnection(conn): DatabaseConnection,
    JSONValidate(payload): JSONValidate<RegisterEmail>,
) -> Result<impl IntoResponse> {
    let user: User = create_user(
        conn,
        payload.email.as_deref().unwrap(),
        Some(payload.password.as_deref().unwrap()),
        payload.first_name.as_deref(),
        payload.last_name.as_deref(),
    )
    .await?;
    Ok(Json(user).into_response())
}

#[debug_handler(state=ConnectionPool)]
pub async fn user_list(
    DatabaseConnection(conn): DatabaseConnection,
    QueryValidate(filter): QueryValidate<UserQuery>,
    QueryValidate(pagination): QueryValidate<PaginationOptions>,
) -> Result<impl IntoResponse> {
    let (query, mut query_param) =
        filter.as_sql_string("ILIKE", "AND", "id DESC");

    let mut query_str =
        "select id, email, image, username, first_name, last_name, \
    is_active, create_at, update_at, last_login from users "
            .to_string()
            + query.as_str();
    let (rows, has_next) = Builder::query(
        &conn,
        &mut query_str,
        &mut query_param,
        None,
        Some(&pagination),
    )
    .await?;

    // println!("Query: {}", query_str);
    // println!("Query Param: {:?}", query_param);
    let users: Vec<User> = rows
        [..rows.len().min(pagination.limit.unwrap() as usize)]
        .iter()
        .map(|row| User {
            id: Some(Cow::Owned(row.get(0))),
            email: Some(row.get(1)),
            image: row.get(2),
            username: Some(Cow::Owned(row.get(3))),
            first_name: row.get(4),
            last_name: row.get(5),
            is_active: Some(row.get(6)),
            create_at: row.get(7),
            update_at: row.get(8),
            last_login: row.get(9),
        })
        .collect();

    Ok(Json(ListResponse {
        data: users,
        pagination: PaginationOptions {
            next: None,
            has_next: Some(has_next),
            limit: pagination.limit,
        },
    })
    .into_response())
}
