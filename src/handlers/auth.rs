use actix_web::{post,web, Error, HttpResponse, HttpRequest, HttpMessage};
use actix_web::error::{ErrorUnauthorized, ErrorInternalServerError};
use actix_web::http::header::AUTHORIZATION;
use jsonwebtoken::{ decode, Algorithm, DecodingKey, Validation, TokenData };
use mongodb::Database;
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};

use crate::models::user::User;
use crate::utils::hash::{hash_password, verify_password};
use crate::utils::jwt::{create_jwt, Claims, extract_email_from_jwt};

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

pub async fn register_user(
    db: web::Data<Database>, 
    form: web::Json<RegisterRequest>
) -> HttpResponse {
    let collection = db.collection::<User>("users");

    // check for existing user
    if let Ok(Some(_)) = collection.find_one(doc! { "email": &form.email }).await {
        return HttpResponse::BadRequest().body("Email already exists");
    }

    // Hash password
    let password_hash = match hash_password(&form.password) {
        Ok(hash) => hash,
        Err(_) => {
            return HttpResponse::InternalServerError().body("Failed to hash password");
        }
    };

    let new_user = User {
        id: None,
        username: form.username.clone(),
        email: form.email.clone(),
        password: password_hash,
        refresh_token: None,
    };

    match collection.insert_one(new_user).await {
        Ok(_) => HttpResponse::Ok().body("User registered successfully"),
        Err(_) => HttpResponse::InternalServerError().body("Failed to register user"),
    }
}

// pub async fn login_user(
//     db: web::Data<Database>,
//     form: web::Json<LoginRequest>,
// ) -> HttpResponse {
//     let collection = db.collection::<User>("users");

//     let user = match collection.find_one(doc! { "email": &form.email }).await {
//         Ok(Some(user)) => user,
//         _ => {
//             return HttpResponse::Unauthorized().body("Invalid email or password");
//         }
//     };

//     // verify password
//     match verify_password(&user.password_hash, &form.password) {
//         Ok(true) => {
//             match create_jwt(&user.email) {
//                 Ok(token) => HttpResponse::Ok().json(serde_json::json!({ "token": token })),
//                 Err(_) => HttpResponse::InternalServerError().body("JWT generation failed"),
//             }
//         }
//         _ => HttpResponse::Unauthorized().body("Invalid email or password"),
//     }
// }

//-- added Token Refrech Login
//   + On login: receive both access and refresh tokens
//   + When the access token expires, send the refresh token to get a new access token
// pub async fn login_user(db: web::Data<Database>, form: web::Json<LoginRequest>) -> HttpResponse {
//     let collection = db.collection::<User>("users");

//     let user = match collection.find_one(doc! { "email": &form.email }).await {
//         Ok(Some(user)) => user,
//         _ => {
//             return HttpResponse::Unauthorized().body("Invalid email or password");
//         }
//     };

//     // verify password
//     match verify_password(&user.password_hash, &form.password) {
//         Ok(true) => {
//             match create_jwt(&user.email, 5, "access") {
//                 Ok(token) => HttpResponse::Ok().json(serde_json::json!({ "token": token })),
//                 Err(_) => HttpResponse::InternalServerError().body("JWT generation failed"),
//             }
//         }
//         _ => HttpResponse::Unauthorized().body("Invalid email or password"),
//     }
// }

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[post("/login")]
pub async fn login(
    db: web::Data<Database>, 
    credentials: web::Json<LoginRequest>
) -> Result<HttpResponse, Error> {
    let collection = db.collection::<User>("users");

    let user = collection
        .find_one(doc! { "email": &credentials.email }).await
        .map_err(|e| ErrorInternalServerError(format!("Database error: {}", e)))? 
        .ok_or_else(|| ErrorUnauthorized("Invalide credentials in database"))?;

    // validate passsword (user argon2 vefificatiaon)
    if !verify_password(&user.password, &credentials.password)
        .map_err(|_| ErrorUnauthorized("Invalid credentials in verify_password"))? 
    {
        return Err(ErrorUnauthorized("Invalid credentials in verify_password"));
    }

    let access_token = create_jwt(&user.email, 15, "access")
        .map_err(|e| ErrorInternalServerError(format!("Token generation error: {}", e)))?;

    let new_refresh_token = create_jwt(&user.email, 60 * 24 * 7, "refresh")
        .map_err(|e| ErrorInternalServerError(format!("Token generation error: {}", e)))?;

    // Save refresh token
    collection.update_one(
            doc! { "email": &user.email },
            doc! { "$set": { "refresh_token": &new_refresh_token }}
        )
        .await
        .map_err(|e| ErrorInternalServerError(format!("Failed to save refresh token: {}", e)))?;

    Ok(
        HttpResponse::Ok().json(TokenResponse { 
            access_token,
            refresh_token: new_refresh_token,
        })
    )
}


// Protected route
pub async fn get_profile(req: HttpRequest) -> HttpResponse {
    if let Some(claims) = req.extensions().get::<Claims>() {
        HttpResponse::Ok().json(serde_json::json!({ 
            "email": claims.sub,
            "message": "Your are authorized. This is a protected route"
         }))
    } else {
        HttpResponse::Unauthorized().body("Unauthorized")
    }
}

#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct TokenResponse {
    access_token: String,
    refresh_token: String,
}

#[post("/refresh")]
async fn refresh_token(
    db: web::Data<Database>,
    payload: web::Json<RefreshRequest>
) -> Result<HttpResponse, Error> {
    let secret = std::env::var("JWT_SECRET").unwrap();

    let decoded_data: TokenData<Claims> = decode::<Claims>(
            &payload.refresh_token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::new(Algorithm::HS256)
        )
        .map_err(|_| ErrorUnauthorized("Invalid refresh token"))?;

    if decoded_data.claims.token_type != "refresh" {
        return Err(ErrorUnauthorized("Not a refresh token"));
    }

    let collection = db.collection::<User>("users");
    let user = collection
        .find_one(doc! { "email": &decoded_data.claims.sub }).await
        .map_err(|e| ErrorInternalServerError(format!("Database error: {}", e)))?
        .ok_or_else(|| ErrorUnauthorized("Invalid credentials"))?;

    if user.refresh_token.as_deref() != Some(&payload.refresh_token) {
        return Err(ErrorUnauthorized("Refresh token mismatch"));
    }

    let new_access_token = create_jwt(&user.email, 15, "access")
        .map_err(|e| ErrorInternalServerError(format!("Token generation error: {}", e)))?;

    let new_refresh_token = create_jwt(&user.email, 60 * 24 * 7, "refresh")
        .map_err(|e| ErrorInternalServerError(format!("Token generation error: {}", e)))?;

    // update stored refresh token
    collection.update_one(
            doc! { "email": &user.email },
            doc! { "$set": { "refresh_token": &new_refresh_token }}
        )
        .await
        .map_err(|e| ErrorInternalServerError(format!("Failed to save refresh token: {}", e)))?;

    Ok(
        HttpResponse::Ok().json(TokenResponse { 
            access_token: new_access_token,
            refresh_token: new_refresh_token,
        })
    )
}

#[post("/logout")]
async fn logout(
    db: web::Data<Database>,
    req: HttpRequest
) -> Result<HttpResponse, Error> {
    let user_email = get_email_from_request(&req)?;
    println!("-> handlers/auth.rs - logout - user_email: {:?}", user_email);

    let collection = db.collection::<User>("users");
    collection.update_one(
        doc! { "email": &user_email },
        doc! { "$unset": { "refresh_token": "" }}
    )
    .await
    .map_err(|e| ErrorInternalServerError(format!("Database update failed: {}", e)))?;
    
    Ok(HttpResponse::Ok().body("Logged out successfully"))
}

fn get_email_from_request(req: &HttpRequest) -> Result<String, Error> {
    let token = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|auth_header| auth_header.strip_prefix("Bearer "))
        .ok_or_else(||
            actix_web::error::ErrorUnauthorized("Missingor invalid authorization header")
        )?;

    extract_email_from_jwt(token)
}
