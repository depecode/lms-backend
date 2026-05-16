use chrono::{Utc, Duration};
use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation, TokenData};
use serde::{Deserialize, Serialize};
use std::env;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,          // subject: user id
    pub exp: i64,           // expiration (as seconds since epoch)
    pub iat: i64,           // issued at
    pub email: String,      // optional - include for convenience
}

// create token for a user id and email
pub fn create_jwt(user_id: Uuid, email: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let exp_hours: i64 = env::var("JWT_EXP_HOURS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(24);

    let now = Utc::now();
    let exp = (now + Duration::hours(exp_hours)).timestamp();

    let claims = Claims {
        sub: user_id,
        iat: now.timestamp(),
        exp,
        email: email.to_owned(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
}

// validate token and return claims
pub fn decode_jwt(token: &str) -> Result<TokenData<Claims>, jsonwebtoken::errors::Error> {
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    )
}