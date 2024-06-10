use crate::common::response::PaginationOptions;
pub use macros::ToSqlString;

pub trait ToSqlString {
    fn as_sql_string(
        &self,
        operator: &str,
        separator: &str,
        order_by: &str,
        pagination: &PaginationOptions,
    ) -> (String, Vec<String>);
}
