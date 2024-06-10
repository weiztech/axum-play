use base62;
use once_cell::sync::Lazy;
use regex::Regex;
use std::env;
use uuid::Uuid;

pub static EMAIL_SUFFIX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\.[a-zA-Z]{2,}$").unwrap());

pub static PASSWORD_ITERATION: Lazy<u32> = Lazy::new(|| {
    env::var("PASSWORD_ITERATION")
        .unwrap_or_else(|_| "10000".to_string())
        .parse::<u32>()
        .unwrap()
});

pub fn uuid7_b62() -> String {
    base62::encode(Uuid::now_v7().as_u128())
}

pub mod Password {
    use pbkdf2::password_hash::{PasswordVerifier, SaltString};
    use pbkdf2::{
        password_hash::{PasswordHash, PasswordHasher, Salt},
        Params, Pbkdf2,
    };

    pub fn generate_password_hash(
        password: &str,
        salt_str: &str,
        rounds: u32,
    ) -> Option<String> {
        let salt = SaltString::encode_b64(salt_str.as_bytes()).unwrap();
        let params = Params {
            rounds,
            output_length: 32,
        };
        let hash = Pbkdf2.hash_password_customized(
            password.as_bytes(),
            None,
            None,
            params,
            &salt,
        );
        hash.ok().map(|x| x.to_string())
    }

    pub fn is_valid(password: &str, password_hash: &str) -> bool {
        let hash = PasswordHash::new(password_hash).unwrap();
        Pbkdf2.verify_password(password.as_bytes(), &hash).is_ok()
    }
}
