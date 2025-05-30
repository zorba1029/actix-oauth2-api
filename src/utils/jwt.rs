use jsonwebtoken::{
    encode, decode, DecodingKey, EncodingKey, Header, Validation, Algorithm, TokenData
};
use jsonwebtoken::errors::Error as JwtError;
use actix_web::error::ErrorUnauthorized;
use actix_web::Error;
use serde::{Serialize, Deserialize};
use chrono::{Utc, Duration};
use std::env;
use utoipa::ToSchema;

// const SECRET: &[u8] = b"your-secret_key_change_me"; // 사용되지 않으므로 주석 처리 또는 삭제

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "sub": "user@example.com",
    "exp": 1678886400,
    "token_type": "access"
}))]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub token_type: String,  // "access" or "refresh"
}

//-- added Token Refrech Login
//   + On login: receive both access and refresh tokens
//   + When the access token expires, send the refresh token to get a new access token
pub fn create_jwt(user_id: &str, minutes: i64, token_type: &str) -> Result<String, JwtError> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::minutes(minutes))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: user_id.to_string(),
        exp: expiration,
        token_type: token_type.to_owned(),
    };

    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");

    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))
}

pub fn verify_jwt(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    println!("-> utils/jwt.rs - verify_jwt - token: {:?}", token);
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");

    match decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    ) {
        Ok(token_data) => {
            // println!("utils/jwt.rs - verify_jwt - token_data: {:?}", token_data);
            println!("-> utils/jwt.rs - verify_jwt - token_data.claims: {:?}", token_data.claims);
            Ok(token_data.claims)
        }
        Err(e) => {
            eprintln!("Error decoding JWT: {:?}", e);
            eprintln!("Error kind: {:?}", e.kind());
            Err(e)
        }
    }
}

pub fn extract_email_from_jwt(token: &str) -> Result<String, Error> {
    let secret = env::var("JWT_SECRET")
        .map_err(|_| ErrorUnauthorized("Missing JWT_SECRET"))?;

    let token_data: TokenData<Claims> = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    )
    .map_err(|_| ErrorUnauthorized("Invalid or expired token"))?;

    Ok(token_data.claims.sub)
}