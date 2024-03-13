use serde::{Deserialize};
use chrono::{DateTime, Utc};

#[derive(Deserialize, Debug)]
struct User {
    id: u64,
    email: String,
    image: Option<String>,
    slug: String,
    password: String,
    first_name: Option<String>,
    last_name: Option<String>,
    create_at: DateTime<Utc>,
    update_at: Option<DateTime<Utc>>,
    last_login: Option<DateTime<Utc>>,
}
