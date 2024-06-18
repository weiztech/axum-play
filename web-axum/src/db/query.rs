use crate::common::response::PaginationOptions;
use crate::db::extractors::ConnectionPooled;
use tokio_postgres::{Error, Row};

static DEFAULT_PAGINATION: u8 = 10;
static DEFAULT_ORDER_BY: &'static str = "ORDER BY id DESC ";

pub struct Builder;

type QueryResult<T> = Result<T, Error>;

impl Builder {
    pub async fn query(
        con: &ConnectionPooled,
        query: &mut String,
        query_value: &mut Vec<String>,
        order_by: Option<&str>,
        pagination: Option<&PaginationOptions>,
    ) -> QueryResult<(Vec<Row>, bool)> {
        let mut counter = query_value.len();
        let mut pagination_limit = DEFAULT_PAGINATION;
        if let Some(value) = pagination {
            if let Some(next) = value.next.as_ref() {
                *query += format!(
                    " {} id < ${} ",
                    if counter == 0 { "WHERE" } else { "AND" },
                    counter + 1,
                )
                .as_str();
                query_value.push(next.to_string());
            }

            if let Some(limit) = value.limit {
                pagination_limit = limit;
            }
        }

        if let Some(order_by) = order_by {
            *query += format!("ORDER BY {} ", order_by).as_str();
        } else {
            *query += DEFAULT_ORDER_BY;
        }

        *query += format!("LIMIT {}", pagination_limit + 1).as_str();

        let sql_value: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> =
            query_value
                .iter()
                .map(|x| x as &(dyn tokio_postgres::types::ToSql + Sync))
                .collect();

        let rows = con.query(query.as_str(), &sql_value).await?;
        let has_next = if rows.len() > pagination_limit as usize {
            true
        } else {
            false
        };
        Ok((rows, has_next))
    }
}
