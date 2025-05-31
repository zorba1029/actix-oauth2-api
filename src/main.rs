use actix_web::{App, HttpServer, Responder, middleware::Logger, web};
use dotenv::dotenv;
use std::env;
use utoipa::Modify;
use utoipa::OpenApi;
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa_swagger_ui::SwaggerUi;

mod config;
mod handlers;
mod middleware;
mod models;
mod utils;

use config::connect_db;
use handlers::auth::{get_profile, login, logout, refresh_token, register_user};
use middleware::jwt_auth::AuthMiddleware;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Default::default);
        components.add_security_scheme(
            "bearer_auth",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .description(Some("Enter JWT token."))
                    .build(),
            ),
        )
    }
}

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::auth::register_user,
        handlers::auth::login,
        handlers::auth::refresh_token,
        handlers::auth::get_profile,
        handlers::auth::logout
    ),
    components(
        schemas(
            handlers::auth::RegisterRequest,
            handlers::auth::LoginRequest,
            handlers::auth::RefreshRequest,
            handlers::auth::TokenResponse,
            handlers::auth::ProfileResponse,
            utils::jwt::Claims
        )
    ),
    modifiers(&SecurityAddon),
    security(
        (), // 빈 security requirement - 선택적으로 만들기 위해
        ("bearer_auth" = []) // bearer_auth 옵션 제공
    ),
    tags(
        (name = "Authentication", description = "User authentication and token management endpoints")
    ),
    info(
        title = "Rust OAuth2 API with Actix",
        version = "0.1.0",
        description = "An example API demonstrating OAuth2 with JWT in Actix Web.",
        contact(
            name = "Your Name",
            email = "your.email@example.com"
        ),
        license(
            name = "MIT",
            url = "http://opensource.org/licenses/MIT"
        )
    ),
    servers(
        (url = "http://localhost:8080", description = "Local development server")
    )
)]
struct ApiDoc;

async fn index() -> impl Responder {
    "Rust/Actix OAuth2 API is running..."
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
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", ApiDoc::openapi())
                    .config(
                        utoipa_swagger_ui::Config::default()
                            .persist_authorization(true)
                            .display_operation_id(false),
                    ),
            )
            .route("/", web::get().to(index))
            .route("/register", web::post().to(register_user))
            .service(login)
            .service(refresh_token)
            .service(
                web::scope("/api")
                    .wrap(AuthMiddleware)
                    .route("/profile", web::get().to(get_profile)),
            )
            .service(logout)
    })
    .bind(("0.0.0.0", port.parse().unwrap()))?
    .run()
    .await
}
