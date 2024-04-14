use once_cell::sync::Lazy;
use regex::Regex;

pub static EMAIL_SUFFIX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\.[a-zA-Z]{2,}$").unwrap());
