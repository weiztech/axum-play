use axum::{debug_handler, response::IntoResponse, Json};
use serde_json::Value;
use std::borrow::Cow;

use crate::common::error::Result;
use crate::common::extractor::{JSONValidate, QueryValidate};
use crate::common::response::{ListResponse, PaginationOptions};
use crate::db::extractors::{ConnectionPool, DatabaseConnection};
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
    let (query, query_param) =
        filter.as_sql_string("ILIKE", "AND", "id DESC", &pagination);

    let sql_query = format!("\
        select id, email, image, username, first_name, last_name, is_active, create_at, \
        update_at, last_login from users {} ", query);
    let sql_value: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> =
        query_param
            .iter()
            .map(|val| val as &(dyn tokio_postgres::types::ToSql + Sync))
            .collect();

    let rows = conn.query(sql_query.as_str(), &sql_value).await?;
    let users: Vec<User> = rows
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

    let total_user = users.len() as u32;
    let last_user_id = users.last().unwrap().id.as_ref().unwrap().to_string();
    Ok(Json(ListResponse {
        items: users,
        pagination: PaginationOptions {
            previous: None,
            next_previous: if total_user == pagination.limit.unwrap() {
                Some(last_user_id)
            } else {
                None
            },
            limit: pagination.limit,
        },
    })
    .into_response())
}
