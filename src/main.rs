use actix_web::{middleware::Logger, web, App, HttpServer, Responder};
use dotenv::dotenv;
use std::env;

mod config;
mod handlers;
mod utils;
mod models;
mod middleware;

use config::connect_db;
use handlers::auth::{login, register_user, get_profile, refresh_token, logout};
use middleware::jwt_auth::AuthMiddleware;

async fn index() -> impl Responder {
    "Rust OAuth2 API is running..."
}

//-----------------------------
// https://www.djamware.com/post/6836f7bc3069a919de614b05/rest-api-security-with-rust-mongodb-and-oauth2
//-----------------------------

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let db = connect_db().await;

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(db.clone()))
            .route("/", web::get().to(index))
            .route("/register", web::post().to(register_user))
            .service(login)
            .service(refresh_token)
            .service(
                web::scope("/api")
                    .wrap(AuthMiddleware)
                    .route("/profile", web::get().to(get_profile))
            )
            .service(logout)
    })
    .bind(("127.0.0.1", port.parse().unwrap()))?
    .run()
    .await
}

