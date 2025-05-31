use actix_web::error::{ErrorInternalServerError, ErrorUnauthorized};
use actix_web::http::header::AUTHORIZATION;
use actix_web::{Error, HttpMessage, HttpRequest, HttpResponse, post, web};
use jsonwebtoken::{Algorithm, DecodingKey, TokenData, Validation, decode};
use mongodb::Database;
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::models::user::User;
use crate::utils::hash::{hash_password, verify_password};
use crate::utils::jwt::{Claims, create_jwt, extract_email_from_jwt};

#[derive(Debug, Deserialize, ToSchema)]
#[schema(example = json!({
    "username": "djamware",
    "email": "admin@djamware.com",
    "password": "mypassword"
}))]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

//-----------------------------------------
/// Register new user
#[utoipa::path(
    post,
    path = "/register",
    operation_id = "register",
    security(), // 빈 security - 인증 불필요
    request_body = RegisterRequest,
    responses(
        (status = 200, description = "User registered successfully", body = String, example = json!("User registered successfully")),
        (status = 400, description = "Email already exists", body = String, example = json!("Email already exists")),
        (status = 500, description = "Failed to hash password or register user", body = String, example = json!("Failed to hash password"))
    )
)]
pub async fn register_user(
    db: web::Data<Database>,
    form: web::Json<RegisterRequest>,
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

#[derive(Debug, Deserialize, ToSchema)]
#[schema(example = json!({
    "email": "admin@djamware.com",
    "password": "mypassword"
}))]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

//-----------------------------------------
/// User login
#[utoipa::path(
    post,
    path = "/login",
    operation_id = "login",
    security(), // 빈 security - 인증 불필요
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful, returns access and refresh tokens", body = TokenResponse),
        (status = 401, description = "Invalid credentials"),
        (status = 500, description = "Database or token generation error")
    )
)]
#[post("/login")]
pub async fn login(
    db: web::Data<Database>,
    credentials: web::Json<LoginRequest>,
) -> Result<HttpResponse, Error> {
    let collection = db.collection::<User>("users");

    let user = collection
        .find_one(doc! { "email": &credentials.email })
        .await
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
    collection
        .update_one(
            doc! { "email": &user.email },
            doc! { "$set": { "refresh_token": &new_refresh_token }},
        )
        .await
        .map_err(|e| ErrorInternalServerError(format!("Failed to save refresh token: {}", e)))?;

    Ok(HttpResponse::Ok().json(TokenResponse {
        access_token,
        refresh_token: new_refresh_token,
    }))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize, ToSchema)]
#[schema(example = json!({
    "access_token": "your_access_token_here",
    "refresh_token": "your_refresh_token_here"
}))]
pub struct TokenResponse {
    access_token: String,
    refresh_token: String,
}

//-----------------------------------------
/// Refresh access token
#[utoipa::path(
    post,
    path = "/refresh",
    operation_id = "refresh",
    security(), // 빈 security - 인증 불필요  
    request_body = RefreshRequest,
    responses(
        (status = 200, description = "Token refresh successful", body = TokenResponse),
        (status = 401, description = "Invalid refresh token or refresh token mismatch")
    )
)]
#[post("/refresh")]
pub async fn refresh_token(
    db: web::Data<Database>,
    payload: web::Json<RefreshRequest>,
) -> Result<HttpResponse, Error> {
    let secret = std::env::var("JWT_SECRET").unwrap();

    let decoded_data: TokenData<Claims> = decode::<Claims>(
        &payload.refresh_token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    )
    .map_err(|_| ErrorUnauthorized("Invalid refresh token"))?;

    if decoded_data.claims.token_type != "refresh" {
        return Err(ErrorUnauthorized("Not a refresh token"));
    }

    let collection = db.collection::<User>("users");
    let user = collection
        .find_one(doc! { "email": &decoded_data.claims.sub })
        .await
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
    collection
        .update_one(
            doc! { "email": &user.email },
            doc! { "$set": { "refresh_token": &new_refresh_token }},
        )
        .await
        .map_err(|e| ErrorInternalServerError(format!("Failed to save refresh token: {}", e)))?;

    Ok(HttpResponse::Ok().json(TokenResponse {
        access_token: new_access_token,
        refresh_token: new_refresh_token,
    }))
}

#[derive(Serialize, ToSchema)]
#[schema(example = json!({"email": "user@example.com", "message": "Your are authorized. This is a protected route"}))]
pub struct ProfileResponse {
    email: String,
    message: String,
}

//-----------------------------------------
/// Get user profile
#[utoipa::path(
    get,
    path = "/api/profile",
    operation_id = "profile",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Profile data", body = ProfileResponse),
        (status = 401, description = "Unauthorized - Missing or invalid token")
    )
)]
// Protected route
pub async fn get_profile(req: HttpRequest) -> HttpResponse {
    if let Some(claims) = req.extensions().get::<Claims>() {
        HttpResponse::Ok().json(ProfileResponse {
            email: claims.sub.clone(),
            message: "Your are authorized. This is a protected route".to_string(),
        })
    } else {
        HttpResponse::Unauthorized().body("Unauthorized")
    }
}

//-----------------------------------------
/// User logout  
#[utoipa::path(
    post,
    path = "/logout",
    operation_id = "logout",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Logged out successfully", body = String, example = json!("Logged out successfully")),
        (status = 401, description = "Unauthorized - Missing or invalid token"),
        (status = 500, description = "Internal server error")
    )
)]
#[post("/logout")]
pub async fn logout(db: web::Data<Database>, req: HttpRequest) -> Result<HttpResponse, Error> {
    let user_email = get_email_from_request(&req)?;
    println!(
        "-> handlers/auth.rs - logout - user_email: {:?}",
        user_email
    );

    let collection = db.collection::<User>("users");
    collection
        .update_one(
            doc! { "email": &user_email },
            doc! { "$unset": { "refresh_token": "" }},
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
        .ok_or_else(|| {
            actix_web::error::ErrorUnauthorized("Missingor invalid authorization header")
        })?;

    extract_email_from_jwt(token)
}
