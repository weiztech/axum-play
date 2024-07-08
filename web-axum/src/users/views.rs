use crate::common::error::{AppError, Result};
use crate::common::extractor::{JSONValidate, QueryValidate};
use crate::common::response::{ErrorResponse, ListResponse, PaginationOptions};
use crate::db::extractors::{ConnectionPool, DatabaseConnection};
use crate::db::query::Builder;
use crate::users::schema::{
    ProfileChange, RegisterEmail, UserPasswordLogin, UserQuery,
};
use crate::users::{db::create_user, models::User};
use axum::{
    debug_handler, extract::Path, http::StatusCode, response::IntoResponse,
    Json,
};
use std::borrow::Cow;
use tokio_postgres::types::ToSql;
use tokio_postgres::GenericClient;

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
            id: Some(Cow::Borrowed(row.get(0))),
            email: Some(row.get(1)),
            image: row.get(2),
            username: Some(Cow::Borrowed(row.get(3))),
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

#[debug_handler(state=ConnectionPool)]
pub async fn edit_user(
    DatabaseConnection(conn): DatabaseConnection,
    Path(user_id): Path<String>,
    JSONValidate(payload): JSONValidate<ProfileChange>,
) -> Result<impl IntoResponse> {
    if user_id.len() < 20 {
        return Err(AppError::from(ErrorResponse::create_error(
            "Invalid user id",
        )));
    }

    let is_valid_user = conn
        .query_one(
            "SELECT EXISTS(SELECT 1 FROM users WHERE id=$1)",
            &[&user_id],
        )
        .await?;
    let value: bool = is_valid_user.get(0);

    if !value {
        return Err(AppError::NotFound("User not found".to_string()));
    }

    let mut idx = 1;
    let mut fields = String::new();
    let mut user_data: Vec<Box<(dyn ToSql + Sync + Send)>> = Vec::new();
    if let Some(first_name) = payload.first_name.as_ref() {
        fields += format!("first_name = ${}", idx).as_str();
        idx += 1;
        user_data.push(if first_name.is_none() {
            Box::new(first_name)
        } else {
            Box::new(first_name.as_deref().unwrap())
        });
    };

    if let Some(last_name) = payload.last_name {
        fields += format!(
            "{}last_name = ${}",
            if fields.is_empty() { "" } else { ", " },
            idx
        )
        .as_str();
        idx += 1;
        user_data.push(if last_name.is_none() {
            Box::new(last_name)
        } else {
            Box::new(last_name.unwrap())
        });
    }

    user_data.push(Box::new(&user_id));
    let query = format!(
        "Update users SET {} WHERE id = ${} RETURNING id, email, image, \
    username, first_name, last_name, is_active, create_at, update_at, last_login ",
        fields, idx
    );

    let query_params: Vec<&(dyn ToSql + Sync)> = user_data
        .iter()
        .map(|p| p.as_ref() as &(dyn ToSql + Sync))
        .collect();
    let row = conn.query_one(query.as_str(), &query_params).await?;
    let user = User {
        id: Some(Cow::Borrowed(row.get(0))),
        email: Some(row.get(1)),
        image: row.get(2),
        username: Some(Cow::Borrowed(row.get(3))),
        first_name: row.get(4),
        last_name: row.get(5),
        is_active: Some(row.get(6)),
        create_at: row.get(7),
        update_at: row.get(8),
        last_login: row.get(9),
    };

    Ok(Json(user).into_response())
}

#[debug_handler(state=ConnectionPool)]
pub async fn delete_user(
    DatabaseConnection(conn): DatabaseConnection,
    Path(user_id): Path<String>,
) -> Result<impl IntoResponse> {
    if user_id.len() < 20 {
        return Err(AppError::from(ErrorResponse::create_error(
            "Invalid user id",
        )));
    }

    let is_deleted: u64 = conn
        .execute("DELETE FROM users WHERE id=$1", &[&user_id])
        .await?;

    if is_deleted > 0 {
        return Ok(StatusCode::NO_CONTENT.into_response());
    }
    Ok(StatusCode::NOT_FOUND.into_response())
}
